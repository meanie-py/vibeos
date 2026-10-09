use crate::comctl32::combo::COMBO_FlipListbox;
use crate::comctl32::button::set_control_clipping;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_listbox);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const WC_LISTBOXW: [::core::ffi::c_ushort; 8] = unsafe {
    ::core::mem::transmute::<[u8; 16], [::core::ffi::c_ushort; 8]>(*b"L\0i\0s\0t\0B\0o\0x\0\0\0")
};

static mut LISTBOX_Timer: TIMER_DIRECTION = LB_TIMER_NONE;

// WINE_DEFAULT_DEBUG_CHANNEL(listbox): debug-channel static, omitted (no Rust equivalent needed here)

pub const LB_ARRAY_GRANULARITY: i32 = 16;
// listbox.c:1944 `#define ATTRIBS (FILE_ATTRIBUTE_READONLY | ..HIDDEN | ..SYSTEM | ..ARCHIVE)`
const ATTRIBS: u32 = (FILE_ATTRIBUTE_READONLY | FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM
    | FILE_ATTRIBUTE_ARCHIVE) as u32;

pub const LB_SCROLL_TIMEOUT: i32 = 50;

pub const LB_TIMER_ID: i32 = 2;

pub const LBS_DISPLAYCHANGED: u32 = 0x80000000;

#[repr(C)]
pub struct LB_ITEMDATA {
    pub str: LPWSTR,
    pub selected: BOOL,
    pub height: UINT,
    pub data: ULONG_PTR,
}
pub type PLB_ITEMDATA = *mut LB_ITEMDATA;

#[repr(C)]
pub union LB_DESCR_U {
    pub items: *mut LB_ITEMDATA,
    pub nodata_items: *mut BYTE,
}

#[repr(C)]
pub struct LB_DESCR {
    pub self_: HWND,
    pub owner: HWND,
    pub style: UINT,
    pub width: INT,
    pub height: INT,
    pub u: LB_DESCR_U,
    pub nb_items: INT,
    pub items_size: UINT,
    pub top_item: INT,
    pub selected_item: INT,
    pub focus_item: INT,
    pub anchor_item: INT,
    pub item_height: INT,
    pub page_size: INT,
    pub column_width: INT,
    pub horz_extent: INT,
    pub horz_pos: INT,
    pub nb_tabs: INT,
    pub tabs: *mut INT,
    pub avg_char_width: INT,
    pub wheel_remain: INT,
    pub caret_on: BOOL,
    pub captured: BOOL,
    pub in_focus: BOOL,
    pub font: HFONT,
    pub locale: LCID,
    pub lphc: *mut HEADCOMBO,
}

// IS_OWNERDRAW, HAS_STRINGS, IS_MULTISELECT, SEND_NOTIFICATION, ISWIN31: function-like / expression macros, inlined by the functions

pub type TIMER_DIRECTION = i32;
pub const LB_TIMER_NONE: TIMER_DIRECTION = 0;
pub const LB_TIMER_UP: TIMER_DIRECTION = 1;
pub const LB_TIMER_LEFT: TIMER_DIRECTION = 2;
pub const LB_TIMER_DOWN: TIMER_DIRECTION = 3;
pub const LB_TIMER_RIGHT: TIMER_DIRECTION = 4;

// static LRESULT LISTBOX_GetItemRect(...) — prototype only, no definition here.
// Stray `else { ... } return len; }` fragment: not a file-scope declaration, omitted.
// #ifdef __REACTOS__ block: LISTBOX_Unregister / COMBOLBOX_Unregister are function definitions, omitted.

unsafe extern "C" fn get_sizeof_item(descr: *const LB_DESCR) -> size_t {
    return if ((*descr).style & (LBS_NODATA as UINT)) != 0 {
        ::core::mem::size_of::<BYTE>() as size_t
    } else {
        ::core::mem::size_of::<LB_ITEMDATA>() as size_t
    };
}

unsafe extern "C" fn resize_storage(descr: *mut LB_DESCR, mut items_size: UINT) -> BOOL {
    let mut items: *mut LB_ITEMDATA = ::core::ptr::null_mut::<LB_ITEMDATA>();
    if items_size > (*descr).items_size
        || items_size.wrapping_add((LB_ARRAY_GRANULARITY * 2) as UINT)
            < (*descr).items_size
    {
        items_size = items_size
            .wrapping_add(LB_ARRAY_GRANULARITY as UINT)
            .wrapping_sub(1 as UINT)
            & !((LB_ARRAY_GRANULARITY - 1) as UINT);
        if ((*descr).style & ((LBS_NODATA | LBS_MULTIPLESEL | LBS_EXTENDEDSEL) as UINT))
            != (LBS_NODATA as UINT)
        {
            items = heap_realloc(
                (*descr).u.items as *mut ::core::ffi::c_void,
                (items_size as SIZE_T).wrapping_mul(get_sizeof_item(descr) as SIZE_T),
            ) as *mut LB_ITEMDATA;
            if items.is_null() {
                SendMessageW(
                    (*descr).owner,
                    WM_COMMAND as UINT,
                    ((GetWindowLongPtrW((*descr).self_, GWLP_ID) as DWORD_PTR
                        & 0xffff as DWORD_PTR) as WORD as DWORD
                        | (((LBN_ERRSPACE) as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                            as DWORD)
                            << 16 as ::core::ffi::c_int) as LONG as DWORD
                        as WPARAM,
                    (*descr).self_ as LPARAM,
                );
                return FALSE;
            }
            (*descr).u.items = items;
        }
        (*descr).items_size = items_size;
    }

    if ((*descr).style & (LBS_NODATA as UINT)) != 0
        && !(*descr).u.nodata_items.is_null()
        && items_size > (*descr).nb_items as UINT
    {
        memset(
            (*descr).u.nodata_items.offset((*descr).nb_items as isize) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            (items_size.wrapping_sub((*descr).nb_items as UINT) as size_t)
                .wrapping_mul(get_sizeof_item(descr)),
        );
    }
    return TRUE;
}

unsafe extern "C" fn get_item_data(descr: *const LB_DESCR, index: UINT) -> ULONG_PTR {
    return if (*descr).style & LBS_NODATA as UINT != 0 {
        0 as ULONG_PTR
    } else {
        (*(*descr).u.items.offset(index as isize)).data
    };
}

unsafe extern "C" fn set_item_data(descr: *mut LB_DESCR, index: UINT, data: ULONG_PTR) {
    if ((*descr).style & LBS_NODATA as UINT) == 0 {
        (*(*descr).u.items.offset(index as isize)).data = data;
    }
}

unsafe extern "C" fn get_item_string(descr: *const LB_DESCR, index: UINT) -> *mut WCHAR {
    return if ((*descr).style & (LBS_OWNERDRAWFIXED | LBS_OWNERDRAWVARIABLE) as UINT) == 0
        || ((*descr).style & LBS_HASSTRINGS as UINT) != 0
    {
        (*(*descr).u.items.offset(index as isize)).str as *mut WCHAR
    } else {
        ::core::ptr::null_mut::<WCHAR>()
    };
}

unsafe extern "C" fn set_item_string(descr: *const LB_DESCR, index: UINT, string: *mut WCHAR) {
    if ((*descr).style & (LBS_NODATA as UINT)) == 0 {
        (*(*descr).u.items.offset(index as isize)).str = string as LPWSTR;
    }
}

unsafe extern "C" fn get_item_height(descr: *const LB_DESCR, index: UINT) -> UINT {
    return if ((*descr).style & (LBS_NODATA as UINT)) != 0 {
        0 as UINT
    } else {
        (*(*descr).u.items.offset(index as isize)).height
    };
}

unsafe extern "C" fn set_item_height(descr: *mut LB_DESCR, index: UINT, height: UINT) {
    if ((*descr).style & (LBS_NODATA as UINT)) == 0 {
        (*(*descr).u.items.offset(index as isize)).height = height;
    }
}

unsafe extern "C" fn is_item_selected(descr: *const LB_DESCR, index: UINT) -> BOOL {
    if ((*descr).style & (LBS_MULTIPLESEL | LBS_EXTENDEDSEL)) == 0 {
        return (index == ((*descr).selected_item as UINT)) as BOOL;
    }
    if ((*descr).style & LBS_NODATA) != 0 {
        return *(*descr).u.nodata_items.add(index as usize) as BOOL;
    } else {
        return (*(*descr).u.items.add(index as usize)).selected;
    }
}

unsafe extern "C" fn set_item_selected_state(descr: *mut LB_DESCR, index: UINT, state: BOOL) {
    if (*descr).style & (LBS_MULTIPLESEL | LBS_EXTENDEDSEL) as UINT != 0 {
        if (*descr).style & LBS_NODATA as UINT != 0 {
            *(*descr).u.nodata_items.offset(index as isize) = state as BYTE;
        } else {
            (*(*descr).u.items.offset(index as isize)).selected = state;
        }
    }
}

unsafe extern "C" fn insert_item_data(descr: *mut LB_DESCR, index: UINT) {
    let size: size_t = get_sizeof_item(descr);
    let p: *mut BYTE = (*descr).u.nodata_items.add(index as usize * size);

    if (*descr).u.items.is_null() {
        return;
    }

    if index < (*descr).nb_items as UINT {
        memmove(
            p.add(size) as *mut core::ffi::c_void,
            p as *const core::ffi::c_void,
            ((*descr).nb_items as UINT - index) as usize * size,
        );
    }
}

unsafe extern "C" fn remove_item_data(descr: *mut LB_DESCR, index: UINT) {
    let size: size_t = get_sizeof_item(descr);
    let p: *mut BYTE = (*descr).u.nodata_items.add((index as size_t).wrapping_mul(size));

    if (*descr).u.items.is_null() {
        return;
    }

    if index < (*descr).nb_items as UINT {
        memmove(
            p as *mut core::ffi::c_void,
            p.add(size) as *const core::ffi::c_void,
            (((*descr).nb_items as UINT).wrapping_sub(index) as size_t).wrapping_mul(size),
        );
    }
}

unsafe extern "C" fn LISTBOX_GetCurrentPageSize(descr: *const LB_DESCR) -> INT {
    let mut i: INT;
    let mut height: INT;
    if ((*descr).style & LBS_OWNERDRAWVARIABLE as UINT) == 0 {
        return (*descr).page_size;
    }
    i = (*descr).top_item;
    height = 0 as INT;
    while i < (*descr).nb_items {
        height = (height as UINT).wrapping_add(get_item_height(descr, i as UINT)) as INT;
        if height > (*descr).height {
            break;
        }
        i += 1;
    }
    if i == (*descr).top_item {
        return 1 as INT;
    } else {
        return i - (*descr).top_item;
    }
}

unsafe extern "C" fn LISTBOX_GetMaxTopIndex(descr: *const LB_DESCR) -> INT {
    let mut max: INT = 0;
    let mut page: INT = 0;
    if ((*descr).style & (LBS_OWNERDRAWVARIABLE as UINT)) != 0 {
        page = (*descr).height;
        max = (*descr).nb_items - 1;
        while max >= 0 {
            page = (page as UINT).wrapping_sub(get_item_height(descr, max as UINT)) as INT;
            if page < 0 {
                break;
            }
            max -= 1;
        }
        if max < (*descr).nb_items - 1 {
            max += 1;
        }
    } else if ((*descr).style & (LBS_MULTICOLUMN as UINT)) != 0 {
        page = (*descr).width / (*descr).column_width;
        if page < 1 {
            page = 1;
        }
        max = ((*descr).nb_items + (*descr).page_size - 1) / (*descr).page_size;
        max = (max - page) * (*descr).page_size;
    } else {
        max = (*descr).nb_items - (*descr).page_size;
    }
    if max < 0 {
        max = 0;
    }
    return max;
}

unsafe extern "C" fn LISTBOX_UpdateScroll(mut descr: *mut LB_DESCR) {
    let mut info: SCROLLINFO = core::mem::zeroed();
    if ((*descr).style & (LBS_NOREDRAW as UINT)) != 0 {
        return;
    }
    info.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    if ((*descr).style & (LBS_MULTICOLUMN as UINT)) != 0 {
        info.nMin = 0 as INT;
        info.nMax = (((*descr).nb_items - (1 as INT)) / (*descr).page_size) as INT;
        info.nPos = ((*descr).top_item / (*descr).page_size) as INT;
        info.nPage = ((*descr).width / (*descr).column_width) as UINT;
        if info.nPage < (1 as UINT) {
            info.nPage = 1 as UINT;
        }
        info.fMask = (SIF_RANGE as UINT) | (SIF_POS as UINT) | (SIF_PAGE as UINT);
        if ((*descr).style & (LBS_DISABLENOSCROLL as UINT)) != 0 {
            info.fMask |= SIF_DISABLENOSCROLL as UINT;
        }
        if ((*descr).style & (WS_HSCROLL as UINT)) != 0 {
            SetScrollInfo(
                (*descr).self_,
                SB_HORZ as INT,
                core::ptr::addr_of_mut!(info) as LPCSCROLLINFO,
                TRUE as BOOL,
            );
        }
        info.nMax = 0 as INT;
        info.fMask = SIF_RANGE as UINT;
        if ((*descr).style & (WS_VSCROLL as UINT)) != 0 {
            SetScrollInfo(
                (*descr).self_,
                SB_VERT as INT,
                core::ptr::addr_of_mut!(info) as LPCSCROLLINFO,
                TRUE as BOOL,
            );
        }
    } else {
        info.nMin = 0 as INT;
        info.nMax = (*descr).nb_items - (1 as INT);
        info.nPos = (*descr).top_item;
        info.nPage = LISTBOX_GetCurrentPageSize(descr) as UINT;
        info.fMask = (SIF_RANGE as UINT) | (SIF_POS as UINT) | (SIF_PAGE as UINT);
        if ((*descr).style & (LBS_DISABLENOSCROLL as UINT)) != 0 {
            info.fMask |= SIF_DISABLENOSCROLL as UINT;
        }
        if ((*descr).style & (WS_VSCROLL as UINT)) != 0 {
            SetScrollInfo(
                (*descr).self_,
                SB_VERT as INT,
                core::ptr::addr_of_mut!(info) as LPCSCROLLINFO,
                TRUE as BOOL,
            );
        }

        if (((*descr).style & (WS_HSCROLL as UINT)) != 0) && (*descr).horz_extent != 0 {
            info.nPos = (*descr).horz_pos as INT;
            info.nPage = (*descr).width as UINT;
            info.fMask = (SIF_POS as UINT) | (SIF_PAGE as UINT);
            if ((*descr).style & (LBS_DISABLENOSCROLL as UINT)) != 0 {
                info.fMask |= SIF_DISABLENOSCROLL as UINT;
            }
            SetScrollInfo(
                (*descr).self_,
                SB_HORZ as INT,
                core::ptr::addr_of_mut!(info) as LPCSCROLLINFO,
                TRUE as BOOL,
            );
        } else {
            if ((*descr).style & (LBS_DISABLENOSCROLL as UINT)) != 0 {
                info.nMin = 0 as INT;
                info.nMax = 0 as INT;
                info.fMask = (SIF_RANGE as UINT) | (SIF_DISABLENOSCROLL as UINT);
                SetScrollInfo(
                    (*descr).self_,
                    SB_HORZ as INT,
                    core::ptr::addr_of_mut!(info) as LPCSCROLLINFO,
                    TRUE as BOOL,
                );
            } else {
                ShowScrollBar(
                    (*descr).self_,
                    SB_HORZ as INT,
                    FALSE as BOOL,
                );
            }
        }
    }
}

unsafe extern "C" fn LISTBOX_SetTopItem(
    mut descr: *mut LB_DESCR,
    mut index: INT,
    mut scroll: BOOL,
) -> LRESULT {
    let mut max: INT = LISTBOX_GetMaxTopIndex(descr);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listbox.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTBOX_SetTopItem\0" as *const u8 as *const ::core::ffi::c_char,
            390 as ::core::ffi::c_int,
            b"setting top item %d, scroll %d\n\0" as *const u8
                as *const ::core::ffi::c_char,
            index,
            scroll,
        );
    }
    if index > max {
        index = max;
    }
    if index < 0 {
        index = 0;
    }
    if ((*descr).style & (LBS_MULTICOLUMN as UINT)) != 0 {
        index -= index % (*descr).page_size;
    }
    if (*descr).top_item == index {
        return LB_OKAY as LRESULT;
    }
    if scroll != 0 {
        let mut dx: INT = 0;
        let mut dy: INT = 0;
        if ((*descr).style & (LBS_MULTICOLUMN as UINT)) != 0 {
            dx = ((*descr).top_item - index) / (*descr).page_size * (*descr).column_width;
        } else if ((*descr).style & (LBS_OWNERDRAWVARIABLE as UINT)) != 0 {
            let mut i: INT = 0;
            if index > (*descr).top_item {
                i = (index as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as INT;
                while i >= (*descr).top_item {
                    dy = (dy as UINT).wrapping_sub(get_item_height(descr, i as UINT)) as INT;
                    i -= 1;
                }
            } else {
                i = index;
                while i < (*descr).top_item {
                    dy = (dy as UINT).wrapping_add(get_item_height(descr, i as UINT)) as INT;
                    i += 1;
                }
            }
        } else {
            dy = ((*descr).top_item - index) * (*descr).item_height;
        }
        ScrollWindowEx(
            (*descr).self_,
            dx,
            dy,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (SW_INVALIDATE | SW_ERASE | SW_SCROLLCHILDREN) as UINT,
        );
    } else {
        InvalidateRect((*descr).self_, core::ptr::null_mut(), TRUE);
    }
    (*descr).top_item = index;
    LISTBOX_UpdateScroll(descr);
    return LB_OKAY as LRESULT;
}

unsafe extern "C" fn LISTBOX_UpdatePage(mut descr: *mut LB_DESCR) {
    let mut page_size: INT = 0;
    if (*descr).item_height == 0 || {
        page_size = (*descr).height / (*descr).item_height;
        page_size < 1
    } {
        page_size = 1;
    }
    if page_size == (*descr).page_size {
        return;
    }
    (*descr).page_size = page_size;
    if (*descr).style & LBS_MULTICOLUMN as UINT != 0 {
        InvalidateRect((*descr).self_, core::ptr::null::<RECT>(), TRUE);
    }
    LISTBOX_SetTopItem(descr, (*descr).top_item, FALSE);
}

unsafe extern "C" fn LISTBOX_UpdateSize(descr: *mut LB_DESCR) {
    let mut rect: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    GetClientRect((*descr).self_, &raw mut rect);
    (*descr).width = (rect.right - rect.left) as INT;
    (*descr).height = (rect.bottom - rect.top) as INT;
    if ((*descr).style & LBS_NOINTEGRALHEIGHT) == 0
        && ((*descr).style & LBS_OWNERDRAWVARIABLE) == 0
    {
        let mut remaining: INT = 0;
        let mut rect: RECT = tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        GetWindowRect((*descr).self_, &raw mut rect);
        if (*descr).item_height != 0 {
            remaining = (*descr).height % (*descr).item_height;
        } else {
            remaining = 0 as INT;
        }
        if ((*descr).height > (*descr).item_height) && remaining != 0 {
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/listbox.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"LISTBOX_UpdateSize\0" as *const u8 as *const ::core::ffi::c_char,
                    475 as ::core::ffi::c_int,
                    b"[%p]: changing height %d -> %d\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    (*descr).self_,
                    (*descr).height,
                    (*descr).height - remaining,
                );
            }
            SetWindowPos(
                (*descr).self_,
                core::ptr::null_mut(),
                0,
                0,
                rect.right - rect.left,
                rect.bottom - rect.top - remaining,
                SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOMOVE,
            );
            return;
        }
    }
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/listbox.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTBOX_UpdateSize\0" as *const u8 as *const ::core::ffi::c_char,
            482 as ::core::ffi::c_int,
            b"[%p]: new size = %d,%d\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*descr).self_,
            (*descr).width,
            (*descr).height,
        );
    }
    LISTBOX_UpdatePage(descr);
    LISTBOX_UpdateScroll(descr);

    /* Invalidate the focused item so it will be repainted correctly */
    if LISTBOX_GetItemRect(descr, (*descr).focus_item, &raw mut rect) == 1 {
        InvalidateRect(
            (*descr).self_,
            &raw mut rect as LPCRECT,
            FALSE,
        );
    }
}

unsafe extern "C" fn LISTBOX_GetItemRect(descr: *const LB_DESCR, index: INT, rect: *mut RECT) -> LRESULT {
    /* Index <= 0 is legal even on empty listboxes */
    if index != 0 && index >= (*descr).nb_items {
        SetRectEmpty(rect);
        SetLastError(ERROR_INVALID_INDEX as DWORD);
        return LB_ERR as LRESULT;
    }
    SetRect(
        rect as LPRECT,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        (*descr).width as ::core::ffi::c_int,
        (*descr).height as ::core::ffi::c_int,
    );
    if ((*descr).style & (LBS_MULTICOLUMN as UINT)) != 0 {
        let mut col: INT = index / (*descr).page_size - (*descr).top_item / (*descr).page_size;
        (*rect).left += (col * (*descr).column_width) as LONG;
        (*rect).right = (*rect).left + (*descr).column_width as LONG;
        (*rect).top += ((index % (*descr).page_size) * (*descr).item_height) as LONG;
        (*rect).bottom = (*rect).top + (*descr).item_height as LONG;
    } else if ((*descr).style & (LBS_OWNERDRAWVARIABLE as UINT)) != 0 {
        let mut i: INT = 0;
        (*rect).right += (*descr).horz_pos as LONG;
        if (index >= 0) && (index < (*descr).nb_items) {
            if index < (*descr).top_item {
                i = (*descr).top_item - 1;
                while i >= index {
                    (*rect).top = ((*rect).top as UINT)
                        .wrapping_sub(get_item_height(descr, i as UINT))
                        as LONG;
                    i -= 1;
                }
            } else {
                i = (*descr).top_item;
                while i < index {
                    (*rect).top = ((*rect).top as UINT)
                        .wrapping_add(get_item_height(descr, i as UINT))
                        as LONG;
                    i += 1;
                }
            }
            (*rect).bottom = ((*rect).top as UINT)
                .wrapping_add(get_item_height(descr, index as UINT)) as LONG;
        }
    } else {
        (*rect).top += ((index - (*descr).top_item) * (*descr).item_height) as LONG;
        (*rect).bottom = (*rect).top + (*descr).item_height as LONG;
        (*rect).right += (*descr).horz_pos as LONG;
    }

    return ((*rect).left < (*descr).width
        && (*rect).right > 0
        && (*rect).top < (*descr).height
        && (*rect).bottom > 0) as LRESULT;
}

unsafe extern "C" fn LISTBOX_GetItemFromPoint(
    descr: *const LB_DESCR,
    x: INT,
    y: INT,
) -> INT {
    let mut index: INT = (*descr).top_item;
    if (*descr).nb_items == 1 as INT {
        return -(1 as INT);
    }
    if ((*descr).style & (LBS_OWNERDRAWVARIABLE as UINT)) != 0 {
        let mut pos: INT = 0 as INT;
        if y >= 0 as INT {
            while index < (*descr).nb_items {
                pos = (pos as UINT).wrapping_add(get_item_height(descr, index as UINT)) as INT;
                if pos > y {
                    break;
                }
                index += 1;
            }
        } else {
            while index > 0 as INT {
                index -= 1;
                pos = (pos as UINT).wrapping_sub(get_item_height(descr, index as UINT)) as INT;
                if pos <= y {
                    break;
                }
            }
        }
    } else if ((*descr).style & (LBS_MULTICOLUMN as UINT)) != 0 {
        if y >= (*descr).item_height * (*descr).page_size {
            return -(1 as INT);
        }
        if y >= 0 as INT {
            index += y / (*descr).item_height;
        }
        if x >= 0 as INT {
            index += (x / (*descr).column_width) * (*descr).page_size;
        } else {
            index -= (((x + 1 as INT) / (*descr).column_width)
                - 1 as INT)
                * (*descr).page_size;
        }
    } else {
        index += y / (*descr).item_height;
    }
    if index < 0 as INT {
        return 0 as INT;
    }
    if index >= (*descr).nb_items {
        return -(1 as INT);
    }
    return index;
}

unsafe extern "C" fn LISTBOX_PaintItem(
    mut descr: *mut LB_DESCR,
    mut hdc: HDC,
    mut rect: *const RECT,
    mut index: INT,
    mut action: UINT,
    mut ignoreFocus: BOOL,
) {
    let mut selected: BOOL = FALSE as BOOL;
    let mut focused: BOOL;
    let mut item_str: *mut WCHAR = core::ptr::null_mut::<WCHAR>();

    if index < (*descr).nb_items {
        item_str = get_item_string(descr, index as UINT);
        if ((*descr).style & LBS_NOSEL as UINT) == 0 {
            selected = is_item_selected(descr, index as UINT);
        }
    }

    focused = (ignoreFocus == 0
        && (*descr).focus_item == index
        && (*descr).caret_on != 0
        && (*descr).in_focus != 0) as ::core::ffi::c_int as BOOL;

    if IS_OWNERDRAW!(descr) {
        let mut dis: DRAWITEMSTRUCT = core::mem::zeroed();
        let mut r: RECT = core::mem::zeroed();
        let mut hrgn: HRGN = core::ptr::null_mut::<HRGN__>();

        if index >= (*descr).nb_items {
            if action == ODA_FOCUS as UINT {
                DrawFocusRect(hdc, rect as LPCRECT);
            } else if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int)
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/listbox.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"LISTBOX_PaintItem\0" as *const u8 as *const ::core::ffi::c_char,
                    633 as ::core::ffi::c_int,
                    b"called with an out of bounds index %d(%d) in owner draw, Not good.\n\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    index,
                    (*descr).nb_items,
                );
            }
            return;
        }

        GetClientRect((*descr).self_, &raw mut r);
        hrgn = set_control_clipping(hdc, &raw mut r);

        dis.CtlType = ODT_LISTBOX as UINT;
        dis.CtlID = GetWindowLongPtrW((*descr).self_, GWLP_ID) as UINT;
        dis.hwndItem = (*descr).self_;
        dis.itemAction = action;
        dis.hDC = hdc;
        dis.itemID = index as UINT;
        dis.itemState = 0 as UINT;
        if selected != 0 {
            dis.itemState |= ODS_SELECTED as UINT;
        }
        if focused != 0 {
            dis.itemState |= ODS_FOCUS as UINT;
        }
        if IsWindowEnabled((*descr).self_) == 0 {
            dis.itemState |= ODS_DISABLED as UINT;
        }
        dis.itemData = get_item_data(descr, index as UINT);
        dis.rcItem = *rect;

        SendMessageW(
            (*descr).owner,
            WM_DRAWITEM as UINT,
            dis.CtlID as WPARAM,
            &raw mut dis as LPARAM,
        );
        SelectClipRgn(hdc, hrgn);
        if !hrgn.is_null() {
            DeleteObject(hrgn as HGDIOBJ);
        }
    } else {
        let mut oldText: COLORREF = 0 as COLORREF;
        let mut oldBk: COLORREF = 0 as COLORREF;

        if action == ODA_FOCUS as UINT {
            DrawFocusRect(hdc, rect as LPCRECT);
            return;
        }
        if selected != 0 {
            oldBk = SetBkColor(
                hdc,
                GetSysColor((COLOR_HIGHLIGHT as UINT) as i32) as COLORREF,
            );
            oldText = SetTextColor(
                hdc,
                GetSysColor((COLOR_HIGHLIGHTTEXT as UINT) as i32) as COLORREF,
            );
        } else {
            let mut br: HBRUSH = GetCurrentObject(hdc, OBJ_BRUSH as UINT) as HBRUSH;
            FillRect(hdc, rect as LPCRECT, br);
        }

        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"dll/win32/comctl32/listbox.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"LISTBOX_PaintItem\0" as *const u8 as *const ::core::ffi::c_char,
                689 as ::core::ffi::c_int,
                b"[%p]: painting %d (%s) action=%02x rect=%s\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                (*descr).self_,
                index,
                debugstr_w(item_str),
                action,
                wine_dbgstr_rect(rect),
            );
        }

        if item_str.is_null() {
            ExtTextOutW(
                hdc,
                (*rect).left + 1 as ::core::ffi::c_int,
                (*rect).top,
                (ETO_OPAQUE | ETO_CLIPPED) as UINT,
                rect,
                core::ptr::null::<WCHAR>(),
                0 as UINT,
                core::ptr::null::<INT>(),
            );
        } else if ((*descr).style & LBS_USETABSTOPS as UINT) == 0 {
            // #else (non-__REACTOS__) branch: compiled out, kept for fidelity.
            if false {
                ExtTextOutW(
                    hdc,
                    (*rect).left + 1 as ::core::ffi::c_int,
                    (*rect).top,
                    (ETO_OPAQUE | ETO_CLIPPED) as UINT,
                    rect,
                    item_str,
                    lstrlenW(item_str) as UINT,
                    core::ptr::null::<INT>(),
                );
            }

            let mut rc: RECT = *rect;
            if selected == 0 {
                let mut sz: SIZE = core::mem::zeroed();
                GetTextExtentPoint32(
                    hdc,
                    item_str as LPCWSTR,
                    lstrlenW(item_str as LPCWSTR),
                    &raw mut sz,
                );
                rc.right = min(sz.cx, rc.right);
            }
            ExtTextOutW(
                hdc,
                (*rect).left + 1 as ::core::ffi::c_int,
                (*rect).top,
                (ETO_OPAQUE | ETO_CLIPPED) as UINT,
                &raw mut rc,
                item_str as LPCWSTR,
                lstrlenW(item_str as LPCWSTR) as UINT,
                core::ptr::null::<INT>(),
            );
        } else {
            ExtTextOutW(
                hdc,
                (*rect).left + 1 as ::core::ffi::c_int,
                (*rect).top,
                (ETO_OPAQUE | ETO_CLIPPED) as UINT,
                rect,
                core::ptr::null::<WCHAR>(),
                0 as UINT,
                core::ptr::null::<INT>(),
            );
            TabbedTextOutW(
                hdc,
                (*rect).left + 1 as ::core::ffi::c_int,
                (*rect).top,
                item_str as LPCWSTR,
                lstrlenW(item_str as LPCWSTR) as ::core::ffi::c_int,
                (*descr).nb_tabs,
                (*descr).tabs,
                0 as ::core::ffi::c_int,
            );
        }

        if selected != 0 {
            SetBkColor(hdc, oldBk);
            SetTextColor(hdc, oldText);
        }
        if focused != 0 {
            DrawFocusRect(hdc, rect as LPCRECT);
        }
    }
}

unsafe extern "C" fn LISTBOX_SetRedraw(descr: *mut LB_DESCR, on: BOOL) {
    if on != 0 {
        if (*descr).style & LBS_NOREDRAW == 0 {
            return;
        }
        (*descr).style &= !LBS_NOREDRAW;
        if (*descr).style & LBS_DISPLAYCHANGED != 0 {
            /* page was changed while setredraw false, refresh automatically */
            InvalidateRect((*descr).self_, core::ptr::null_mut(), TRUE);
            if ((*descr).top_item + (*descr).page_size) > (*descr).nb_items {
                /* reset top of page if less than number of items/page */
                (*descr).top_item = (*descr).nb_items - (*descr).page_size;
                if (*descr).top_item < 0 {
                    (*descr).top_item = 0;
                }
            }
            (*descr).style &= !LBS_DISPLAYCHANGED;
        }
        LISTBOX_UpdateScroll(descr);
    } else {
        (*descr).style |= LBS_NOREDRAW;
    }
}

unsafe extern "C" fn LISTBOX_RepaintItem(descr: *mut LB_DESCR, index: INT, action: UINT) {
    let mut hdc: HDC = core::ptr::null_mut();
    let mut rect: RECT = core::mem::zeroed();
    let mut oldFont: HFONT = core::ptr::null_mut();
    let mut hbrush: HBRUSH = core::ptr::null_mut();
    let mut oldBrush: HBRUSH = core::ptr::null_mut();

    /* Do not repaint the item if the item is not visible */
    if IsWindowVisible((*descr).self_) == 0 {
        return;
    }
    if ((*descr).style & LBS_NOREDRAW) != 0 {
        (*descr).style |= LBS_DISPLAYCHANGED;
        return;
    }
    if LISTBOX_GetItemRect(descr, index, &raw mut rect) != 1 {
        return;
    }
    hdc = GetDCEx(
        (*descr).self_,
        core::ptr::null_mut(),
        DCX_CACHE,
    );
    if hdc.is_null() {
        return;
    }
    if !(*descr).font.is_null() {
        oldFont = SelectObject(hdc, (*descr).font as HGDIOBJ) as HFONT;
    }
    hbrush = SendMessageW(
        (*descr).owner,
        WM_CTLCOLORLISTBOX,
        hdc as WPARAM,
        (*descr).self_ as LPARAM,
    ) as HBRUSH;
    if !hbrush.is_null() {
        oldBrush = SelectObject(hdc, hbrush as HGDIOBJ) as HBRUSH;
    }
    if IsWindowEnabled((*descr).self_) == 0 {
        SetTextColor(
            hdc,
            GetSysColor((COLOR_GRAYTEXT) as i32) as COLORREF,
        );
    }
    SetWindowOrgEx(
        hdc,
        (*descr).horz_pos,
        0,
        core::ptr::null_mut(),
    );
    LISTBOX_PaintItem(
        descr,
        hdc,
        &raw mut rect,
        index,
        action,
        TRUE,
    );
    if !oldFont.is_null() {
        SelectObject(hdc, oldFont as HGDIOBJ);
    }
    if !oldBrush.is_null() {
        SelectObject(hdc, oldBrush as HGDIOBJ);
    }
    ReleaseDC((*descr).self_, hdc);
}

unsafe extern "C" fn LISTBOX_DrawFocusRect(mut descr: *mut LB_DESCR, mut on: BOOL) {
    let mut hdc: HDC = core::ptr::null_mut::<HDC__>();
    let mut rect: RECT = core::mem::zeroed::<RECT>();
    let mut oldFont: HFONT = core::ptr::null_mut();
    /* Do not repaint the item if the item is not visible */
    if IsWindowVisible((*descr).self_) == 0 {
        return;
    }
    if (*descr).focus_item == -1 {
        return;
    }
    if (*descr).caret_on == 0 || (*descr).in_focus == 0 {
        return;
    }
    if LISTBOX_GetItemRect(descr, (*descr).focus_item, &raw mut rect) != 1 {
        return;
    }
    hdc = GetDCEx(
        (*descr).self_,
        core::ptr::null_mut(),
        DCX_CACHE,
    );
    if hdc.is_null() {
        return;
    }
    if !(*descr).font.is_null() {
        oldFont = SelectObject(hdc, (*descr).font as HGDIOBJ) as HFONT;
    }
    if IsWindowEnabled((*descr).self_) == 0 {
        SetTextColor(
            hdc,
            GetSysColor(COLOR_GRAYTEXT as INT) as COLORREF,
        );
    }
    SetWindowOrgEx(
        hdc,
        (*descr).horz_pos as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        core::ptr::null_mut::<tagPOINT>(),
    );
    LISTBOX_PaintItem(
        descr,
        hdc,
        &raw mut rect,
        (*descr).focus_item,
        ODA_FOCUS as UINT,
        (on == 0) as ::core::ffi::c_int,
    );
    if !oldFont.is_null() {
        SelectObject(hdc, oldFont as HGDIOBJ);
    }
    ReleaseDC((*descr).self_, hdc);
}

unsafe extern "C" fn LISTBOX_InitStorage(mut descr: *mut LB_DESCR, mut nb_items: INT) -> LRESULT {
    let mut new_size: UINT = ((*descr).nb_items + nb_items) as UINT;
    if new_size > (*descr).items_size && resize_storage(descr, new_size) == 0 {
        return LB_ERRSPACE as LRESULT;
    }
    return (*descr).items_size as LRESULT;
}

unsafe extern "C" fn LISTBOX_SetTabStops(descr: *mut LB_DESCR, count: INT, tabs: LPINT) -> BOOL {
    let mut i: INT;

    if !((*descr).style & LBS_USETABSTOPS != 0) {
        SetLastError(ERROR_LB_WITHOUT_TABSTOPS);
        return FALSE;
    }

    HeapFree(GetProcessHeap(), 0, (*descr).tabs as LPVOID);
    (*descr).nb_tabs = count;
    if (*descr).nb_tabs == 0 {
        (*descr).tabs = core::ptr::null_mut();
        return TRUE;
    }
    (*descr).tabs = HeapAlloc(
        GetProcessHeap(),
        0,
        ((*descr).nb_tabs as usize * core::mem::size_of::<INT>()) as SIZE_T,
    ) as *mut INT;
    if (*descr).tabs.is_null() {
        return FALSE;
    }
    memcpy(
        (*descr).tabs as *mut core::ffi::c_void,
        tabs as *const core::ffi::c_void,
        (*descr).nb_tabs as usize * core::mem::size_of::<INT>(),
    );

    // convert into "dialog units"
    i = 0;
    while i < (*descr).nb_tabs {
        *(*descr).tabs.offset(i as isize) = MulDiv(
            *(*descr).tabs.offset(i as isize),
            (*descr).avg_char_width,
            4,
        );
        i += 1;
    }

    return TRUE;
}

unsafe extern "C" fn LISTBOX_GetText(
    mut descr: *mut LB_DESCR,
    mut index: INT,
    mut buffer: LPWSTR,
    mut unicode: BOOL,
) -> LRESULT {
    let mut len: DWORD = 0;
    if index < 0 as ::core::ffi::c_int || index >= (*descr).nb_items {
        SetLastError(ERROR_INVALID_INDEX as DWORD);
        return LB_ERR as LRESULT;
    }
    if (*descr).style & (LBS_OWNERDRAWFIXED | LBS_OWNERDRAWVARIABLE) as UINT == 0
        || (*descr).style & LBS_HASSTRINGS as UINT != 0
    {
        let mut str: *mut WCHAR = get_item_string(descr, index as UINT);
        if buffer.is_null() {
            return lstrlenW(str as LPCWSTR) as LRESULT;
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
                b"dll/win32/comctl32/listbox.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"LISTBOX_GetText\0" as *const u8 as *const ::core::ffi::c_char,
                886 as ::core::ffi::c_int,
                b"index %d (0x%04x) %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                index,
                index,
                debugstr_w(str),
            );
        }
        lstrcpyW(buffer, str as LPCWSTR);
        len = lstrlenW(buffer as LPCWSTR) as DWORD;
        if _SEH2_Volatile0 != 0
            || 0 as ::core::ffi::c_int != 0
                && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint
        {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/listbox.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"LISTBOX_GetText\0" as *const u8 as *const ::core::ffi::c_char,
                    895 as ::core::ffi::c_int,
                    b"got an invalid buffer (Delphi bug?)\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            return LB_ERR as LRESULT;
        }
    } else {
        if !buffer.is_null() {
            *(buffer as *mut ULONG_PTR) = get_item_data(descr, index as UINT);
        }
        len = ::core::mem::size_of::<ULONG_PTR>() as DWORD;
    }
    return len as LRESULT;
}

unsafe extern "C" fn LISTBOX_lstrcmpiW(mut lcid: LCID, mut str1: LPCWSTR, mut str2: LPCWSTR) -> INT {
    let mut ret: INT = CompareStringW(lcid, NORM_IGNORECASE, str1, -1, str2, -1) as INT;
    if ret == (CSTR_LESS_THAN as i32) {
        return -1;
    }
    if ret == (CSTR_EQUAL as i32) {
        return 0;
    }
    if ret == (CSTR_GREATER_THAN as i32) {
        return 1;
    }
    return -1;
}

unsafe extern "C" fn LISTBOX_FindStringPos(
    descr: *mut LB_DESCR,
    str: LPCWSTR,
    exact: BOOL,
) -> INT {
    let mut index: INT;
    let mut min: INT;
    let mut max: INT;
    let mut res: INT;

    if (*descr).nb_items == 0 || ((*descr).style & (LBS_SORT as UINT)) == 0 {
        return -(1 as INT); /* Add it at the end */
    }

    min = 0 as INT;
    max = (*descr).nb_items - 1 as INT;
    while min <= max {
        index = (min + max) / (2 as INT);
        if HAS_STRINGS!(descr) {
            res = LISTBOX_lstrcmpiW(
                (*descr).locale,
                get_item_string(descr, index as UINT),
                str,
            );
        } else {
            let mut cis: COMPAREITEMSTRUCT = core::mem::zeroed();
            let id: UINT = GetWindowLongPtrW((*descr).self_, GWLP_ID as INT) as UINT;

            cis.CtlType = ODT_LISTBOX as UINT;
            cis.CtlID = id;
            cis.hwndItem = (*descr).self_;
            /* note that some application (MetaStock) expects the second item
             * to be in the listbox */
            cis.itemID1 = index as UINT;
            cis.itemData1 = get_item_data(descr, index as UINT);
            cis.itemID2 = (-(1 as INT)) as UINT;
            cis.itemData2 = str as ULONG_PTR;
            cis.dwLocaleId = (*descr).locale as DWORD;
            res = SendMessageW(
                (*descr).owner,
                WM_COMPAREITEM as UINT,
                id as WPARAM,
                core::ptr::addr_of_mut!(cis) as LPARAM,
            ) as INT;
        }
        if res == 0 {
            return index;
        }
        if res > (0 as INT) {
            max = index - 1 as INT;
        } else {
            min = index + 1 as INT;
        }
    }
    return if exact != 0 {
        -(1 as INT)
    } else {
        min
    };
}

unsafe extern "C" fn LISTBOX_FindFileStrPos(mut descr: *mut LB_DESCR, mut str: LPCWSTR) -> INT {
    let mut min: INT = 0;
    let mut max: INT = 0;
    let mut res: INT = 0;
    if !HAS_STRINGS!(descr) {
        return LISTBOX_FindStringPos(descr, str, FALSE);
    }
    min = 0;
    max = (*descr).nb_items;
    while min != max {
        let mut index: INT = (min + max) / 2;
        let mut p: LPCWSTR = get_item_string(descr, index as UINT) as LPCWSTR;
        if *p == '[' as u16 {
            if *str != '[' as u16 {
                res = -1;
            } else if *p.offset(1) == '-' as u16 {
                if *str.offset(1) == '-' as u16 {
                    res = (*str.offset(2) as INT)
                        - (*p.offset(2) as INT);
                } else {
                    res = -1;
                }
            } else {
                if *str.offset(1) == '-' as u16 {
                    res = 1;
                } else {
                    res = LISTBOX_lstrcmpiW((*descr).locale, str, p);
                }
            }
        } else {
            if *str == '[' as u16 {
                res = 1;
            } else {
                res = LISTBOX_lstrcmpiW((*descr).locale, str, p);
            }
        }
        if res == 0 {
            return index;
        }
        if res < 0 {
            max = index;
        } else {
            min = index + 1;
        }
    }
    return max;
}

unsafe extern "C" fn LISTBOX_FindString(
    mut descr: *mut LB_DESCR,
    mut start: INT,
    mut str: LPCWSTR,
    mut exact: BOOL,
) -> INT {
    let mut i: INT;
    let mut index: INT;
    if ((*descr).style & (LBS_NODATA as UINT)) != 0 {
        return LB_ERR as INT;
    }
    start += 1;
    if start >= (*descr).nb_items {
        start = 0 as INT;
    }
    if (((*descr).style & (LBS_OWNERDRAWFIXED | LBS_OWNERDRAWVARIABLE) as UINT) == 0)
        || (((*descr).style & (LBS_HASSTRINGS as UINT)) != 0)
    {
        if str.is_null() || *str.offset(0 as isize) == 0 {
            return LB_ERR as INT;
        }
        if exact != 0 {
            i = 0 as INT;
            index = start;
            while i < (*descr).nb_items {
                if index == (*descr).nb_items {
                    index = 0 as INT;
                }
                if LISTBOX_lstrcmpiW(
                    (*descr).locale,
                    str,
                    get_item_string(descr, index as UINT) as LPCWSTR,
                ) == 0
                {
                    return index;
                }
                i += 1;
                index += 1;
            }
        } else {
            let mut len: INT = lstrlenW(str) as INT;
            let mut item_str: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
            i = 0 as INT;
            index = start;
            while i < (*descr).nb_items {
                if index == (*descr).nb_items {
                    index = 0 as INT;
                }
                item_str = get_item_string(descr, index as UINT);
                if wcsnicmp(str, item_str, len as usize) == 0 {
                    return index;
                }
                if *item_str.offset(0 as isize) as i32 == '[' as i32 {
                    if wcsnicmp(
                        str,
                        item_str.offset(1 as isize),
                        len as usize,
                    ) == 0
                    {
                        return index;
                    }
                    if *item_str.offset(1 as isize) as i32 == '-' as i32
                        && wcsnicmp(
                            str,
                            item_str.offset(2 as isize),
                            len as usize,
                        ) == 0
                    {
                        return index;
                    }
                }
                i += 1;
                index += 1;
            }
        }
    } else {
        if exact != 0 && ((*descr).style & (LBS_SORT as UINT)) != 0 {
            return LISTBOX_FindStringPos(descr, str, TRUE as BOOL);
        }
        i = 0 as INT;
        index = start;
        while i < (*descr).nb_items {
            if index == (*descr).nb_items {
                index = 0 as INT;
            }
            if get_item_data(descr, index as UINT) == str as ULONG_PTR {
                return index;
            }
            i += 1;
            index += 1;
        }
    }
    return LB_ERR as INT;
}

unsafe extern "C" fn LISTBOX_GetSelCount(descr: *const LB_DESCR) -> LRESULT {
    let mut i: INT = 0;
    let mut count: INT = 0;
    if ((*descr).style & LBS_MULTIPLESEL as UINT) == 0 || ((*descr).style & LBS_NOSEL as UINT) != 0 {
        return LB_ERR as LRESULT;
    }
    count = 0 as INT;
    i = count;
    while i < (*descr).nb_items {
        if is_item_selected(descr, i as UINT) != 0 {
            count += 1;
        }
        i += 1;
    }
    return count as LRESULT;
}

unsafe extern "C" fn LISTBOX_GetSelItems( descr: *const LB_DESCR, max: INT, array: LPINT ) -> LRESULT {
    let mut i: INT;
    let mut count: INT;

    if ((*descr).style & (LBS_MULTIPLESEL as UINT)) == 0 {
        return LB_ERR as LRESULT;
    }
    count = 0 as INT;
    i = count;
    while (i < (*descr).nb_items) && (count < max) {
        if is_item_selected(descr, i as UINT) != 0 {
            let fresh16 = count;
            count += 1;
            *array.offset(fresh16 as isize) = i;
        }
        i += 1;
    }
    return count as LRESULT;
}

unsafe extern "C" fn LISTBOX_Paint(descr: *mut LB_DESCR, hdc: HDC) -> LRESULT {
    let mut i: INT = 0;
    let mut col_pos: INT = (*descr).page_size - (1 as INT);
    let mut rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut focusRect: RECT = RECT {
        left: -(1 as LONG),
        top: -(1 as LONG),
        right: -(1 as LONG),
        bottom: -(1 as LONG),
    };
    let mut oldFont: HFONT = 0 as HFONT;
    let mut hbrush: HBRUSH = core::ptr::null_mut();
    let mut oldBrush: HBRUSH = 0 as HBRUSH;

    if ((*descr).style & (LBS_NOREDRAW as UINT)) != 0 {
        return 0 as LRESULT;
    }

    SetRect(
        &raw mut rect,
        0 as INT,
        0 as INT,
        (*descr).width as INT,
        (*descr).height as INT,
    );
    if ((*descr).style & (LBS_MULTICOLUMN as UINT)) != 0 {
        rect.right = (rect.left as INT + (*descr).column_width) as LONG;
    } else if (*descr).horz_pos != 0 {
        SetWindowOrgEx(
            hdc,
            (*descr).horz_pos as INT,
            0 as INT,
            core::ptr::null_mut::<tagPOINT>(),
        );
        rect.right += (*descr).horz_pos as LONG;
    }

    if !(*descr).font.is_null() {
        oldFont = SelectObject(hdc, (*descr).font as HGDIOBJ) as HFONT;
    }
    hbrush = SendMessageW(
        (*descr).owner,
        WM_CTLCOLORLISTBOX as UINT,
        hdc as WPARAM,
        (*descr).self_ as LPARAM,
    ) as HBRUSH;
    if !hbrush.is_null() {
        oldBrush = SelectObject(hdc, hbrush as HGDIOBJ) as HBRUSH;
    }
    if IsWindowEnabled((*descr).self_) == 0 {
        SetTextColor(
            hdc,
            GetSysColor(COLOR_GRAYTEXT as INT) as COLORREF,
        );
    }

    if (*descr).nb_items == 0
        && (*descr).focus_item != -(1 as INT)
        && (*descr).caret_on != 0
        && (*descr).in_focus != 0
    {
        /* Special case for empty listbox: paint focus rect */
        rect.bottom = (rect.top as INT + (*descr).item_height) as LONG;
        ExtTextOutW(
            hdc,
            0 as INT,
            0 as INT,
            (ETO_OPAQUE | ETO_CLIPPED) as UINT,
            &raw mut rect,
            core::ptr::null::<WCHAR>(),
            0 as UINT,
            core::ptr::null::<INT>(),
        );
        LISTBOX_PaintItem(
            descr,
            hdc,
            &raw mut rect,
            (*descr).focus_item,
            ODA_FOCUS as UINT,
            FALSE as BOOL,
        );
        rect.top = rect.bottom;
    }

    /* Paint all the item, regarding the selection
       Focus state will be painted after  */

    i = (*descr).top_item;
    while i < (*descr).nb_items {
        if ((*descr).style & (LBS_OWNERDRAWVARIABLE as UINT)) == 0 {
            rect.bottom = (rect.top as INT + (*descr).item_height) as LONG;
        } else {
            rect.bottom =
                (rect.top as UINT).wrapping_add(get_item_height(descr, i as UINT)) as LONG;
        }

        /* keep the focus rect, to paint the focus item after */
        if i == (*descr).focus_item {
            focusRect = rect;
        }
        rect.bottom = min(rect.bottom as INT, (*descr).height) as LONG;
        LISTBOX_PaintItem(
            descr,
            hdc,
            &raw mut rect,
            i,
            ODA_DRAWENTIRE as UINT,
            TRUE as BOOL,
        );
        rect.top = rect.bottom;

        if ((*descr).style & (LBS_MULTICOLUMN as UINT)) != 0 && col_pos == 0 {
            if IS_OWNERDRAW!(descr) == false {
                /* Clear the bottom of the column */
                if rect.top < (*descr).height {
                    rect.bottom = (*descr).height as LONG;
                    ExtTextOutW(
                        hdc,
                        0 as INT,
                        0 as INT,
                        (ETO_OPAQUE | ETO_CLIPPED) as UINT,
                        &raw mut rect,
                        core::ptr::null::<WCHAR>(),
                        0 as UINT,
                        core::ptr::null::<INT>(),
                    );
                }
            }

            /* Go to the next column */
            rect.left += (*descr).column_width as LONG;
            rect.right += (*descr).column_width as LONG;
            rect.top = 0 as LONG;
            col_pos = (*descr).page_size - (1 as INT);
            if rect.left >= (*descr).width as LONG {
                break;
            }
        } else {
            col_pos -= 1;
            if rect.top >= (*descr).height as LONG {
                break;
            }
        }
        i += 1;
    }

    /* Paint the focus item now */
    if focusRect.top != focusRect.bottom && (*descr).caret_on != 0 && (*descr).in_focus != 0 {
        LISTBOX_PaintItem(
            descr,
            hdc,
            &raw mut focusRect,
            (*descr).focus_item,
            ODA_FOCUS as UINT,
            FALSE as BOOL,
        );
    }

    if IS_OWNERDRAW!(descr) == false {
        /* Clear the remainder of the client area */
        if rect.top < (*descr).height as LONG {
            rect.bottom = (*descr).height as LONG;
            // __REACTOS__ branch (taken)
            if true {
                let br: HBRUSH =
                    GetCurrentObject(hdc, (OBJ_BRUSH as INT) as UINT) as HBRUSH;
                FillRect(hdc, &raw mut rect as LPCRECT, br);
            } else {
                ExtTextOutW(
                    hdc,
                    0 as INT,
                    0 as INT,
                    (ETO_OPAQUE | ETO_CLIPPED) as UINT,
                    &raw mut rect,
                    core::ptr::null::<WCHAR>(),
                    0 as UINT,
                    core::ptr::null::<INT>(),
                );
            }
        }
        if rect.right < (*descr).width as LONG {
            rect.left = rect.right;
            rect.right = (*descr).width as LONG;
            rect.top = 0 as LONG;
            rect.bottom = (*descr).height as LONG;
            // __REACTOS__ branch (taken)
            if true {
                let br_0: HBRUSH =
                    GetCurrentObject(hdc, (OBJ_BRUSH as INT) as UINT) as HBRUSH;
                FillRect(hdc, &raw mut rect as LPCRECT, br_0);
            } else {
                ExtTextOutW(
                    hdc,
                    0 as INT,
                    0 as INT,
                    (ETO_OPAQUE | ETO_CLIPPED) as UINT,
                    &raw mut rect,
                    core::ptr::null::<WCHAR>(),
                    0 as UINT,
                    core::ptr::null::<INT>(),
                );
            }
        }
    }
    if !oldFont.is_null() {
        SelectObject(hdc, oldFont as HGDIOBJ);
    }
    if !oldBrush.is_null() {
        SelectObject(hdc, oldBrush as HGDIOBJ);
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn LISTBOX_NCPaint(descr: *mut LB_DESCR, region: HRGN) {
    let exstyle: DWORD = GetWindowLongW((*descr).self_, GWL_EXSTYLE) as DWORD;
    let theme: HTHEME = GetWindowTheme((*descr).self_);
    let mut cliprgn: HRGN = region;
    let mut cxEdge: ::core::ffi::c_int = 0;
    let mut cyEdge: ::core::ffi::c_int = 0;
    let mut hdc: HDC = ::core::ptr::null_mut::<HDC__>();
    let mut r: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if theme.is_null() || (exstyle & (WS_EX_CLIENTEDGE as DWORD)) == 0 {
        return;
    }
    cxEdge = GetSystemMetrics((SM_CXEDGE) as i32);
    cyEdge = GetSystemMetrics((SM_CYEDGE) as i32);

    GetWindowRect((*descr).self_, core::ptr::addr_of_mut!(r));

    /* New clipping region passed to default proc to exclude border */
    cliprgn = CreateRectRgn(
        r.left as ::core::ffi::c_int + cxEdge,
        r.top as ::core::ffi::c_int + cyEdge,
        r.right as ::core::ffi::c_int - cxEdge,
        r.bottom as ::core::ffi::c_int - cyEdge,
    );
    if region != (1 as HRGN) {
        CombineRgn(cliprgn, cliprgn, region, (RGN_AND as i32));
    }
    OffsetRect(
        core::ptr::addr_of_mut!(r),
        -(r.left as ::core::ffi::c_int),
        -(r.top as ::core::ffi::c_int),
    );

    /* Non-REACTOS path (preprocessor #else branch), kept as dead code */
    if false {
        hdc = GetDCEx(
            (*descr).self_,
            region,
            DCX_WINDOW | DCX_INTERSECTRGN,
        );
        OffsetRect(
            core::ptr::addr_of_mut!(r),
            -(r.left as ::core::ffi::c_int),
            -(r.top as ::core::ffi::c_int),
        );
    }

    /* #ifdef __REACTOS__ branch (active) */
    hdc = GetWindowDC((*descr).self_);
    /* Exclude client part */
    ExcludeClipRect(
        hdc,
        r.left as ::core::ffi::c_int + cxEdge,
        r.top as ::core::ffi::c_int + cyEdge,
        r.right as ::core::ffi::c_int - cxEdge,
        r.bottom as ::core::ffi::c_int - cyEdge,
    );

    if IsThemeBackgroundPartiallyTransparent(
        theme,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    ) != 0
    {
        DrawThemeParentBackground((*descr).self_, hdc, core::ptr::addr_of_mut!(r));
    }
    DrawThemeBackground(
        theme,
        hdc,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        core::ptr::addr_of_mut!(r),
        0 as *const RECT,
    );
    ReleaseDC((*descr).self_, hdc);
}

unsafe extern "C" fn LISTBOX_InvalidateItems(mut descr: *mut LB_DESCR, mut index: INT) {
    let mut rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if LISTBOX_GetItemRect(descr, index, core::ptr::addr_of_mut!(rect)) == 1 as LRESULT {
        if (*descr).style & (LBS_NOREDRAW as UINT) != 0 {
            (*descr).style |= LBS_DISPLAYCHANGED as UINT;
            return;
        }
        rect.bottom = (*descr).height as LONG;
        InvalidateRect((*descr).self_, core::ptr::addr_of_mut!(rect) as LPCRECT, TRUE as BOOL);
        if (*descr).style & (LBS_MULTICOLUMN as UINT) != 0 {
            /* Repaint the other columns */
            rect.left = rect.right;
            rect.right = (*descr).width as LONG;
            rect.top = 0 as LONG;
            InvalidateRect((*descr).self_, core::ptr::addr_of_mut!(rect) as LPCRECT, TRUE as BOOL);
        }
    }
}

unsafe extern "C" fn LISTBOX_InvalidateItemRect(descr: *mut LB_DESCR, index: INT) {
    let mut rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    if LISTBOX_GetItemRect(descr, index, core::ptr::addr_of_mut!(rect)) == 1 {
        InvalidateRect((*descr).self_, core::ptr::addr_of!(rect), TRUE);
    }
}

unsafe extern "C" fn LISTBOX_GetItemHeight(descr: *const LB_DESCR, index: INT) -> LRESULT {
    if ((*descr).style & LBS_OWNERDRAWVARIABLE as UINT) != 0 && (*descr).nb_items > 0 {
        if (index < 0) || (index >= (*descr).nb_items) {
            SetLastError(ERROR_INVALID_INDEX as DWORD);
            return LB_ERR as LRESULT;
        }
        return get_item_height(descr, index as UINT) as LRESULT;
    } else {
        return (*descr).item_height as LRESULT;
    }
}

unsafe extern "C" fn LISTBOX_SetItemHeight(
    descr: *mut LB_DESCR,
    index: INT,
    mut height: INT,
    repaint: BOOL,
) -> LRESULT {
    if height > MAXWORD as INT {
        return -(1 as INT) as LRESULT;
    }

    if height == 0 {
        height = 1 as INT;
    }

    if ((*descr).style & (LBS_OWNERDRAWVARIABLE as UINT)) != 0 {
        if (index < 0 as INT) || (index >= (*descr).nb_items) {
            SetLastError(ERROR_INVALID_INDEX as DWORD);
            return LB_ERR as LRESULT;
        }

        set_item_height(descr, index as UINT, height as UINT);
        LISTBOX_UpdateScroll(descr);
        if repaint != 0 {
            LISTBOX_InvalidateItems(descr, index);
        }
    } else if height != (*descr).item_height {
        (*descr).item_height = height;
        LISTBOX_UpdatePage(descr);
        LISTBOX_UpdateScroll(descr);
        if repaint != 0 {
            InvalidateRect(
                (*descr).self_,
                0 as *const RECT,
                TRUE as BOOL,
            );
        }
    }
    return LB_OKAY as LRESULT;
}

unsafe extern "C" fn LISTBOX_SetHorizontalPos(descr: *mut LB_DESCR, mut pos: INT) {
    let mut diff: INT;

    if pos > (*descr).horz_extent - (*descr).width {
        pos = (*descr).horz_extent - (*descr).width;
    }
    if pos < 0 {
        pos = 0;
    }
    diff = (*descr).horz_pos - pos;
    if diff == 0 {
        return;
    }

    (*descr).horz_pos = pos;
    LISTBOX_UpdateScroll(descr);
    if abs(diff) < (*descr).width {
        let mut rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        /* Invalidate the focused item so it will be repainted correctly */
        if LISTBOX_GetItemRect(descr, (*descr).focus_item, &raw mut rect) == 1 {
            InvalidateRect((*descr).self_, &raw mut rect, TRUE);
        }
        ScrollWindowEx(
            (*descr).self_,
            diff,
            0,
            core::ptr::null(),
            core::ptr::null(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            SW_INVALIDATE | SW_ERASE | SW_SCROLLCHILDREN,
        );
    } else {
        InvalidateRect((*descr).self_, core::ptr::null(), TRUE);
    }
}

unsafe extern "C" fn LISTBOX_SetHorizontalExtent(descr: *mut LB_DESCR, extent: INT) -> LRESULT {
    if ((*descr).style & LBS_MULTICOLUMN as UINT) != 0 {
        return LB_OKAY as LRESULT;
    }
    if extent == (*descr).horz_extent {
        return LB_OKAY as LRESULT;
    }

    (*descr).horz_extent = extent;
    if ((*descr).style & WS_HSCROLL as UINT) != 0 {
        let mut info: SCROLLINFO = tagSCROLLINFO {
            cbSize: 0,
            fMask: 0,
            nMin: 0,
            nMax: 0,
            nPage: 0,
            nPos: 0,
            nTrackPos: 0,
        };
        info.cbSize = ::core::mem::size_of::<SCROLLINFO>() as UINT;
        info.nMin = 0 as INT;
        info.nMax = if (*descr).horz_extent != 0 {
            (*descr).horz_extent - 1 as INT
        } else {
            0 as INT
        };
        info.fMask = SIF_RANGE as UINT;
        if ((*descr).style & LBS_DISABLENOSCROLL as UINT) != 0 {
            info.fMask |= SIF_DISABLENOSCROLL as UINT;
        }
        SetScrollInfo(
            (*descr).self_,
            SB_HORZ as INT,
            core::ptr::addr_of_mut!(info) as LPCSCROLLINFO,
            TRUE as BOOL,
        );
    }
    if (*descr).horz_pos > extent - (*descr).width {
        LISTBOX_SetHorizontalPos(descr, extent - (*descr).width);
    }
    return LB_OKAY as LRESULT;
}

unsafe extern "C" fn LISTBOX_SetColumnWidth(descr: *mut LB_DESCR, column_width: INT) -> LRESULT {
    let mut rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    GetClientRect((*descr).self_, core::ptr::addr_of_mut!(rect));
    (*descr).width = (rect.right - rect.left) as INT;
    (*descr).height = (rect.bottom - rect.top) as INT;
    (*descr).column_width = column_width;

    LISTBOX_UpdatePage(descr);
    LISTBOX_UpdateScroll(descr);
    return (LB_OKAY) as LRESULT;
}

unsafe extern "C" fn LISTBOX_SetFont(descr: *mut LB_DESCR, font: HFONT) -> INT {
    let mut hdc: HDC = core::ptr::null_mut();
    let mut oldFont: HFONT = core::ptr::null_mut();
    let alphabet: *const i8 = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ\0".as_ptr() as *const i8;
    let mut sz: SIZE = core::mem::zeroed();

    (*descr).font = font;

    hdc = GetDCEx((*descr).self_, core::ptr::null_mut(), DCX_CACHE);
    if hdc.is_null() {
        return 16;
    }
    if !font.is_null() {
        oldFont = SelectObject(hdc, font as HGDIOBJ) as HFONT;
    }
    GetTextExtentPointA(hdc, alphabet as LPCSTR, 52, core::ptr::addr_of_mut!(sz));
    if !oldFont.is_null() {
        SelectObject(hdc, oldFont as HGDIOBJ);
    }
    ReleaseDC((*descr).self_, hdc);

    (*descr).avg_char_width = ((sz.cx / 26 + 1) / 2) as INT;
    if IS_OWNERDRAW!(descr) == false {
        LISTBOX_SetItemHeight(descr, 0, sz.cy as INT, FALSE);
    }
    return sz.cy as INT;
}

unsafe extern "C" fn LISTBOX_MakeItemVisible(descr: *mut LB_DESCR, index: INT, fully: BOOL) {
    let mut top: INT = 0;
    if index <= (*descr).top_item {
        top = index;
    } else if (*descr).style & LBS_MULTICOLUMN as UINT != 0 {
        let mut cols: INT = (*descr).width;
        if fully == 0 {
            cols += (*descr).column_width - 1 as INT;
        }
        if cols >= (*descr).column_width {
            cols /= (*descr).column_width;
        } else {
            cols = 1 as INT;
        }
        if index < (*descr).top_item + (*descr).page_size * cols {
            return;
        }
        top = index - (*descr).page_size * (cols - 1 as INT);
    } else if (*descr).style & LBS_OWNERDRAWVARIABLE as UINT != 0 {
        let mut height: INT = (if fully != 0 {
            get_item_height(descr, index as UINT)
        } else {
            1 as UINT
        }) as INT;
        top = index;
        while top > (*descr).top_item {
            height = (height as UINT).wrapping_add(get_item_height(
                descr,
                (top - 1 as INT) as UINT,
            )) as INT;
            if height > (*descr).height {
                break;
            }
            top -= 1;
        }
    } else {
        if index < (*descr).top_item + (*descr).page_size {
            return;
        }
        if fully == 0
            && index == (*descr).top_item + (*descr).page_size
            && (*descr).height > (*descr).page_size * (*descr).item_height
        {
            return;
        }
        top = index - (*descr).page_size + 1 as INT;
    }
    LISTBOX_SetTopItem(descr, top, TRUE);
}

unsafe extern "C" fn LISTBOX_SetCaretIndex(
    mut descr: *mut LB_DESCR,
    mut index: INT,
    mut fully_visible: BOOL,
) -> LRESULT {
    let mut focus_changed: BOOL = ((*descr).focus_item != index) as BOOL;
    if (*descr).style & LBS_NOSEL as UINT != 0 {
        return LB_ERR as LRESULT;
    }
    if index < 0 || index >= (*descr).nb_items {
        return LB_ERR as LRESULT;
    }
    if focus_changed != 0 {
        LISTBOX_DrawFocusRect(descr, FALSE);
        (*descr).focus_item = index;
    }
    LISTBOX_MakeItemVisible(descr, index, fully_visible);
    if focus_changed != 0 {
        LISTBOX_DrawFocusRect(descr, TRUE);
    }
    return LB_OKAY as LRESULT;
}

unsafe extern "C" fn LISTBOX_SelectItemRange(
    descr: *mut LB_DESCR,
    mut first: INT,
    mut last: INT,
    on: BOOL,
) -> LRESULT {
    let mut i: INT = 0;

    /* A few sanity checks */

    if (*descr).style & LBS_NOSEL as UINT != 0 {
        return LB_ERR as LRESULT;
    }
    if !((*descr).style & LBS_MULTIPLESEL as UINT != 0) {
        return LB_ERR as LRESULT;
    }

    if (*descr).nb_items == 0 {
        return LB_OKAY as LRESULT;
    }

    if last == -1 as INT || last >= (*descr).nb_items {
        last = (*descr).nb_items - 1 as INT;
    }
    if first < 0 as INT {
        first = 0 as INT;
    }
    if last < first {
        return LB_OKAY as LRESULT;
    }

    if on != 0 {
        /* Turn selection on */
        i = first;
        while i <= last {
            if !(is_item_selected(descr, i as UINT) != 0) {
                set_item_selected_state(descr, i as UINT, TRUE as BOOL);
                LISTBOX_InvalidateItemRect(descr, i);
            }
            i += 1;
        }
    } else {
        /* Turn selection off */
        i = first;
        while i <= last {
            if !(is_item_selected(descr, i as UINT) == 0) {
                set_item_selected_state(descr, i as UINT, FALSE as BOOL);
                LISTBOX_InvalidateItemRect(descr, i);
            }
            i += 1;
        }
    }
    return LB_OKAY as LRESULT;
}

unsafe extern "C" fn LISTBOX_SetSelection(
    descr: *mut LB_DESCR,
    index: INT,
    on: BOOL,
    send_notify: BOOL,
) -> LRESULT {
    if ((*descr).style & (LBS_NOSEL as UINT)) != 0 {
        (*descr).selected_item = index;
        return LB_ERR as LRESULT;
    }
    if (index < -(1 as INT)) || (index >= (*descr).nb_items) {
        return LB_ERR as LRESULT;
    }
    if ((*descr).style & (LBS_MULTIPLESEL as UINT)) != 0 {
        if index == -(1 as INT) {
            /* Select all items */
            return LISTBOX_SelectItemRange(
                descr,
                0 as INT,
                (*descr).nb_items,
                on,
            );
        } else {
            /* Only one item */
            return LISTBOX_SelectItemRange(descr, index, index, on);
        }
    } else {
        let mut oldsel: INT = (*descr).selected_item;
        if index == oldsel {
            return LB_OKAY as LRESULT;
        }
        if oldsel != -(1 as INT) {
            set_item_selected_state(descr, oldsel as UINT, FALSE as BOOL);
        }
        if index != -(1 as INT) {
            set_item_selected_state(descr, index as UINT, TRUE as BOOL);
        }
        (*descr).selected_item = index;
        if oldsel != -(1 as INT) {
            LISTBOX_RepaintItem(descr, oldsel, ODA_SELECT as UINT);
        }
        if index != -(1 as INT) {
            LISTBOX_RepaintItem(descr, index, ODA_SELECT as UINT);
        }
        if send_notify != 0 && (*descr).nb_items != 0 {
            let code: UINT = if index != -(1 as INT) {
                LBN_SELCHANGE as UINT
            } else {
                LBN_SELCANCEL as UINT
            };
            SendMessageW(
                (*descr).owner,
                WM_COMMAND as UINT,
                MAKEWPARAM!(
                    GetWindowLongPtrW((*descr).self_, GWLP_ID) as WORD,
                    code as WORD,
                ) as WPARAM,
                (*descr).self_ as LPARAM,
            );
        } else if !(*descr).lphc.is_null() {
            (*(*descr).lphc).wState |= CBF_SELCHANGE as UINT;
        }
    }
    return LB_OKAY as LRESULT;
}

unsafe extern "C" fn LISTBOX_MoveCaret(
    descr: *mut LB_DESCR,
    index: INT,
    fully_visible: BOOL,
) {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listbox.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTBOX_MoveCaret\0" as *const u8 as *const ::core::ffi::c_char,
            1624 as ::core::ffi::c_int,
            b"old focus %d, index %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*descr).focus_item,
            index,
        );
    }

    if (index < 0) || (index >= (*descr).nb_items) {
        return;
    }

    /* 1. remove the focus and repaint the item */
    LISTBOX_DrawFocusRect(descr, FALSE);

    /* 2. then turn off the previous selection */
    /* 3. repaint the new selected item */
    if (*descr).style & LBS_EXTENDEDSEL as UINT != 0 {
        if (*descr).anchor_item != -1 {
            let first: INT = min(index, (*descr).anchor_item);
            let last: INT = max(index, (*descr).anchor_item);
            if first > 0 {
                LISTBOX_SelectItemRange(
                    descr,
                    0,
                    first - 1,
                    FALSE,
                );
            }
            LISTBOX_SelectItemRange(
                descr,
                last + 1,
                -1,
                FALSE,
            );
            LISTBOX_SelectItemRange(descr, first, last, TRUE);
        }
    } else if (*descr).style & LBS_MULTIPLESEL as UINT == 0 {
        /* Set selection to new caret item */
        LISTBOX_SetSelection(
            descr,
            index,
            TRUE,
            FALSE,
        );
    }

    /* 4. repaint the new item with the focus */
    (*descr).focus_item = index;
    LISTBOX_MakeItemVisible(descr, index, fully_visible);
    LISTBOX_DrawFocusRect(descr, TRUE);
}

unsafe extern "C" fn LISTBOX_InsertItem(
    descr: *mut LB_DESCR,
    mut index: INT,
    str: LPWSTR,
    data: ULONG_PTR,
) -> LRESULT {
    let oldfocus: INT = (*descr).focus_item;

    if index == -(1) {
        index = (*descr).nb_items;
    } else if (index < 0) || (index > (*descr).nb_items) {
        return LB_ERR as LRESULT;
    }
    if resize_storage(descr, ((*descr).nb_items + 1) as UINT) == 0 {
        return LB_ERR as LRESULT;
    }

    insert_item_data(descr, index as UINT);
    (*descr).nb_items += 1;
    set_item_string(descr, index as UINT, str as *mut WCHAR);
    set_item_data(
        descr,
        index as UINT,
        if HAS_STRINGS!(descr) {
            0
        } else {
            data
        },
    );
    set_item_height(descr, index as UINT, 0);
    set_item_selected_state(descr, index as UINT, FALSE);

    /* Get item height */

    if ((*descr).style & LBS_OWNERDRAWVARIABLE) != 0 {
        let mut mis: MEASUREITEMSTRUCT = core::mem::zeroed();
        let id: UINT = GetWindowLongPtrW((*descr).self_, GWLP_ID) as UINT;

        mis.CtlType = ODT_LISTBOX;
        mis.CtlID = id;
        mis.itemID = index as UINT;
        mis.itemData = data;
        mis.itemHeight = (*descr).item_height as UINT;
        SendMessageW(
            (*descr).owner,
            WM_MEASUREITEM,
            id as WPARAM,
            &raw mut mis as LPARAM,
        );
        set_item_height(
            descr,
            index as UINT,
            if mis.itemHeight != 0 {
                mis.itemHeight
            } else {
                1
            },
        );
    }

    /* Repaint the items */

    LISTBOX_UpdateScroll(descr);
    LISTBOX_InvalidateItems(descr, index);

    /* Move selection and focused item */
    /* If listbox was empty, set focus to the first item */
    if (*descr).nb_items == 1 {
        LISTBOX_SetCaretIndex(descr, 0, FALSE);
    }
    /* single select don't change selection index in win31 */
    else if ((((GetVersion() as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as INT) == 0xa03)
        && !IS_MULTISELECT!(descr))
    {
        (*descr).selected_item += 1;
        LISTBOX_SetSelection(
            descr,
            (*descr).selected_item - 1,
            TRUE,
            FALSE,
        );
    } else {
        if index <= (*descr).selected_item {
            (*descr).selected_item += 1;
            (*descr).focus_item = oldfocus; /* focus not changed */
        }
    }
    return LB_OKAY as LRESULT;
}

unsafe extern "C" fn LISTBOX_InsertString(descr: *mut LB_DESCR, mut index: INT, mut str: LPCWSTR) -> LRESULT {
    let mut new_str: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut ret: LRESULT = 0;
    if HAS_STRINGS!(descr) {
        static mut empty_stringW: [WCHAR; 1] = [0];
        if str.is_null() {
            str = core::ptr::addr_of!(empty_stringW) as LPCWSTR;
        }
        new_str = HeapAlloc(
            GetProcessHeap(),
            0,
            ((lstrlenW(str) + 1) as SIZE_T)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if new_str.is_null() {
            SEND_NOTIFICATION!(descr, LBN_ERRSPACE);
            return (LB_ERRSPACE) as LRESULT;
        }
        lstrcpyW(new_str, str);
    }
    if index == -1 {
        index = (*descr).nb_items;
    }
    ret = LISTBOX_InsertItem(descr, index, new_str, str as ULONG_PTR);
    if ret != 0 {
        HeapFree(GetProcessHeap(), 0, new_str as PVOID);
        return ret;
    }
    if ((*__wine_dbch___default).flags as INT & ((1 as INT) << (__WINE_DBCL_TRACE as INT))) != 0 {
        let __dbch: *mut __wine_debug_channel = *core::ptr::addr_of_mut!(__wine_dbch___default);
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listbox.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTBOX_InsertString\0" as *const u8 as *const ::core::ffi::c_char,
            1758 as ::core::ffi::c_int,
            b"[%p]: added item %d %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*descr).self_,
            index,
            if ((*descr).style & ((16 as INT) | (32 as INT)) as UINT) == 0
                || ((*descr).style & 64 as UINT) != 0
            {
                debugstr_w(new_str as *const WCHAR)
            } else {
                b"\0" as *const u8 as *const ::core::ffi::c_char
            },
        );
    }
    return index as LRESULT;
}

unsafe extern "C" fn LISTBOX_DeleteItem(mut descr: *mut LB_DESCR, mut index: INT) {
    if IS_OWNERDRAW!(descr) != false || get_item_data(descr, index as UINT) != 0 {
        let mut dis: DELETEITEMSTRUCT = core::mem::zeroed();
        let mut id: UINT = GetWindowLongPtrW((*descr).self_, GWLP_ID) as UINT;

        dis.CtlType = ODT_LISTBOX;
        dis.CtlID = id;
        dis.itemID = index as UINT;
        dis.hwndItem = (*descr).self_;
        dis.itemData = get_item_data(descr, index as UINT);
        SendMessageW(
            (*descr).owner,
            WM_DELETEITEM,
            id as WPARAM,
            core::ptr::addr_of_mut!(dis) as LPARAM,
        );
    }
    HeapFree(
        GetProcessHeap(),
        0,
        get_item_string(descr, index as UINT) as PVOID,
    );
}

unsafe extern "C" fn LISTBOX_RemoveItem(mut descr: *mut LB_DESCR, mut index: INT) -> LRESULT {
    if (index < 0) || (index >= (*descr).nb_items) {
        return LB_ERR as LRESULT;
    }

    /* We need to invalidate the original rect instead of the updated one. */
    LISTBOX_InvalidateItems(descr, index);

    if (*descr).nb_items == 1 {
        SendMessageW(
            (*descr).self_,
            LB_RESETCONTENT as UINT,
            0 as WPARAM,
            0 as LPARAM,
        );
        return LB_OKAY as LRESULT;
    }
    (*descr).nb_items -= 1;
    LISTBOX_DeleteItem(descr, index);
    remove_item_data(descr, index as UINT);

    if (*descr).anchor_item == (*descr).nb_items {
        (*descr).anchor_item -= 1;
    }
    resize_storage(descr, (*descr).nb_items as UINT);

    /* Repaint the items */

    LISTBOX_UpdateScroll(descr);
    /* if we removed the scrollbar, reset the top of the list
      (correct for owner-drawn ???) */
    if (*descr).nb_items == (*descr).page_size {
        LISTBOX_SetTopItem(descr, 0 as INT, TRUE as BOOL);
    }

    /* Move selection and focused item */
    if !(((*descr).style & (LBS_MULTIPLESEL | LBS_EXTENDEDSEL) as UINT) != 0
        && ((*descr).style & LBS_NOSEL as UINT) == 0)
    {
        if index == (*descr).selected_item {
            (*descr).selected_item = -(1) as INT;
        } else if index < (*descr).selected_item {
            (*descr).selected_item -= 1;
            /* win 31 do not change the selected item number */
            if ((GetVersion() as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as INT) == 0xa03 {
                LISTBOX_SetSelection(
                    descr,
                    (*descr).selected_item + 1,
                    TRUE as BOOL,
                    FALSE as BOOL,
                );
            }
        }
    }

    if (*descr).focus_item >= (*descr).nb_items {
        (*descr).focus_item = (*descr).nb_items - 1;
        if (*descr).focus_item < 0 {
            (*descr).focus_item = 0;
        }
    }
    return LB_OKAY as LRESULT;
}

unsafe extern "C" fn LISTBOX_ResetContent(descr: *mut LB_DESCR) {
    let mut i: INT = 0;

    if ((*descr).style & LBS_NODATA) == 0 {
        i = (*descr).nb_items - 1;
        while i >= 0 {
            LISTBOX_DeleteItem(descr, i);
            i -= 1;
        }
    }
    HeapFree(GetProcessHeap(), 0, (*descr).u.items as PVOID);
    (*descr).nb_items = 0;
    (*descr).top_item = 0;
    (*descr).selected_item = -1;
    (*descr).focus_item = 0;
    (*descr).anchor_item = -1;
    (*descr).items_size = 0;
    (*descr).u.items = core::ptr::null_mut();
}

unsafe extern "C" fn LISTBOX_SetCount(descr: *mut LB_DESCR, count: UINT) -> LRESULT {
    let orig_num: UINT = (*descr).nb_items as UINT;

    if ((*descr).style & (LBS_NODATA as UINT)) == 0 {
        return LB_ERR as LRESULT;
    }

    if resize_storage(descr, count) == 0 {
        return LB_ERRSPACE as LRESULT;
    }
    (*descr).nb_items = count as INT;

    if count != 0 {
        LISTBOX_UpdateScroll(descr);
        if count < orig_num {
            (*descr).anchor_item =
                (if ((*descr).anchor_item as UINT) < count.wrapping_sub(1 as UINT) {
                    (*descr).anchor_item as UINT
                } else {
                    count.wrapping_sub(1 as UINT)
                }) as INT;
            if (*descr).selected_item as UINT >= count {
                (*descr).selected_item = -(1 as INT);
            }

            /* If we removed the scrollbar, reset the top of the list */
            if count <= (*descr).page_size as UINT && orig_num > (*descr).page_size as UINT {
                LISTBOX_SetTopItem(descr, 0 as INT, TRUE);
            }

            (*descr).focus_item =
                (if ((*descr).focus_item as UINT) < count.wrapping_sub(1 as UINT) {
                    (*descr).focus_item as UINT
                } else {
                    count.wrapping_sub(1 as UINT)
                }) as INT;
        }

        /* If it was empty before growing, set focus to the first item */
        else if orig_num == (0 as UINT) {
            LISTBOX_SetCaretIndex(descr, 0 as INT, FALSE);
        }
    } else {
        SendMessageW(
            (*descr).self_,
            LB_RESETCONTENT as UINT,
            0 as WPARAM,
            0 as LPARAM,
        );
    }

    InvalidateRect((*descr).self_, ::core::ptr::null::<RECT>(), TRUE);
    return LB_OKAY as LRESULT;
}

unsafe extern "C" fn LISTBOX_Directory(
    descr: *mut LB_DESCR,
    attrib: UINT,
    filespec: LPCWSTR,
    long_names: BOOL,
) -> LRESULT {
    let mut handle: HANDLE = core::ptr::null_mut();
    let mut ret: LRESULT = (LB_OKAY) as _;
    let mut entry: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut pos: ::core::ffi::c_int = 0;
    let mut maxinsert: LRESULT = (LB_ERR) as _;

    /* don't scan directory if we just want drives exclusively */
    if attrib != (DDL_DRIVES | DDL_EXCLUSIVE) as UINT {
        /* scan directory */
        handle = FindFirstFileW(filespec, core::ptr::addr_of_mut!(entry));
        if handle == INVALID_HANDLE_VALUE {
            let le: ::core::ffi::c_int = GetLastError() as ::core::ffi::c_int;
            if le != (ERROR_NO_MORE_FILES as i32) && le != (ERROR_FILE_NOT_FOUND as i32) {
                return (LB_ERR) as LRESULT;
            }
        } else {
            'scan: loop {
                'body: {
                    let mut buffer: [WCHAR; 270] = [0; 270];
                    if (entry.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                        static bracketW: [WCHAR; 2] = [0x005Du16, 0x0000];
                        static dotW: [WCHAR; 2] = [0x002Eu16, 0x0000];
                        if (attrib & DDL_DIRECTORY) == 0
                            || lstrcmpW(
                                (core::ptr::addr_of_mut!(entry.cFileName) as *mut WCHAR) as LPCWSTR,
                                core::ptr::addr_of!(dotW) as LPCWSTR,
                            ) == 0
                        {
                            break 'body;
                        }
                        buffer[0] = '[' as i32 as WCHAR;
                        if long_names == 0
                            && entry.cAlternateFileName[0] != 0
                        {
                            lstrcpyW(
                                (core::ptr::addr_of_mut!(buffer) as *mut WCHAR)
                                    .offset(1 as isize),
                                (core::ptr::addr_of_mut!(entry.cAlternateFileName) as *mut WCHAR) as LPCWSTR,
                            );
                        } else {
                            lstrcpyW(
                                (core::ptr::addr_of_mut!(buffer) as *mut WCHAR)
                                    .offset(1 as isize),
                                (core::ptr::addr_of_mut!(entry.cFileName) as *mut WCHAR) as LPCWSTR,
                            );
                        }
                        lstrcatW(
                            core::ptr::addr_of_mut!(buffer) as LPWSTR,
                            core::ptr::addr_of!(bracketW) as LPCWSTR,
                        );
                    } else {
                        /* not a directory */
                        if (attrib & DDL_EXCLUSIVE) != 0
                            && ((attrib & ATTRIBS)
                                != (entry.dwFileAttributes & ATTRIBS))
                        {
                            break 'body;
                        }
                        if long_names == 0
                            && entry.cAlternateFileName[0] != 0
                        {
                            lstrcpyW(
                                core::ptr::addr_of_mut!(buffer) as LPWSTR,
                                (core::ptr::addr_of_mut!(entry.cAlternateFileName) as *mut WCHAR) as LPCWSTR,
                            );
                        } else {
                            lstrcpyW(
                                core::ptr::addr_of_mut!(buffer) as LPWSTR,
                                (core::ptr::addr_of_mut!(entry.cFileName) as *mut WCHAR) as LPCWSTR,
                            );
                        }
                    }
                    if long_names == 0 {
                        CharLowerW(core::ptr::addr_of_mut!(buffer) as LPWSTR);
                    }
                    pos = LISTBOX_FindFileStrPos(
                        descr,
                        core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPCWSTR,
                    ) as ::core::ffi::c_int;
                    ret = LISTBOX_InsertString(
                        descr,
                        pos as INT,
                        core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPCWSTR,
                    );
                    if ret < 0 {
                        break 'scan;
                    }
                    if ret <= maxinsert {
                        maxinsert += 1;
                    } else {
                        maxinsert = ret;
                    }
                }
                /* do-while condition */
                if !(FindNextFileW(handle, core::ptr::addr_of_mut!(entry)) != 0) {
                    break;
                }
            }
            FindClose(handle);
        }
    }
    if ret >= 0 {
        ret = maxinsert;

        /* scan drives */
        if (attrib & DDL_DRIVES) != 0 {
            let mut buffer: [WCHAR; 6] = [0x005Bu16, 0x002D, 0x0061, 0x002D, 0x005D, 0x0000];
            let mut root: [WCHAR; 4] = [0x0041u16, 0x003A, 0x005C, 0x0000];
            let mut drive: ::core::ffi::c_int = 0;
            drive = 0;
            'drives: while drive < 26 {
                'body: {
                    if GetDriveTypeW(core::ptr::addr_of_mut!(root) as *mut WCHAR as LPCWSTR)
                        <= DRIVE_NO_ROOT_DIR
                    {
                        break 'body;
                    }
                    ret = LISTBOX_InsertString(
                        descr,
                        -(1),
                        core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPCWSTR,
                    );
                    if ret < 0 {
                        break 'drives;
                    }
                }
                drive += 1;
                buffer[2] = buffer[2].wrapping_add(1);
                root[0] = root[0].wrapping_add(1);
            }
        }
    }
    return ret;
}

unsafe extern "C" fn LISTBOX_HandleVScroll(
    mut descr: *mut LB_DESCR,
    mut scrollReq: WORD,
    mut pos: WORD,
) -> LRESULT {
    let mut info: SCROLLINFO = tagSCROLLINFO {
        cbSize: 0,
        fMask: 0,
        nMin: 0,
        nMax: 0,
        nPage: 0,
        nPos: 0,
        nTrackPos: 0,
    };
    if ((*descr).style & LBS_MULTICOLUMN as UINT) != 0 {
        return 0 as LRESULT;
    }
    match (scrollReq as ::core::ffi::c_int) as u32 {
        SB_LINEUP => {
            LISTBOX_SetTopItem(descr, (*descr).top_item - 1 as INT, TRUE as BOOL);
        }
        SB_LINEDOWN => {
            LISTBOX_SetTopItem(descr, (*descr).top_item + 1 as INT, TRUE as BOOL);
        }
        SB_PAGEUP => {
            LISTBOX_SetTopItem(
                descr,
                (*descr).top_item - LISTBOX_GetCurrentPageSize(descr),
                TRUE as BOOL,
            );
        }
        SB_PAGEDOWN => {
            LISTBOX_SetTopItem(
                descr,
                (*descr).top_item + LISTBOX_GetCurrentPageSize(descr),
                TRUE as BOOL,
            );
        }
        SB_THUMBPOSITION => {
            LISTBOX_SetTopItem(descr, pos as INT, TRUE as BOOL);
        }
        SB_THUMBTRACK => {
            info.cbSize = ::core::mem::size_of::<SCROLLINFO>() as UINT;
            info.fMask = SIF_TRACKPOS as UINT;
            GetScrollInfo((*descr).self_, SB_VERT as INT, core::ptr::addr_of_mut!(info));
            LISTBOX_SetTopItem(descr, info.nTrackPos as INT, TRUE as BOOL);
        }
        SB_TOP => {
            LISTBOX_SetTopItem(descr, 0 as INT, TRUE as BOOL);
        }
        SB_BOTTOM => {
            LISTBOX_SetTopItem(descr, (*descr).nb_items, TRUE as BOOL);
        }
        _ => {}
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn LISTBOX_HandleHScroll(
    descr: *mut LB_DESCR,
    scrollReq: WORD,
    pos: WORD,
) -> LRESULT {
    let mut info: SCROLLINFO = core::mem::zeroed::<SCROLLINFO>();
    let mut page: INT = 0;

    if ((*descr).style & LBS_MULTICOLUMN) != 0 {
        match (scrollReq as ::core::ffi::c_int) as u32 {
            SB_LINELEFT => {
                LISTBOX_SetTopItem(descr, (*descr).top_item - (*descr).page_size, TRUE);
            }
            SB_LINERIGHT => {
                LISTBOX_SetTopItem(descr, (*descr).top_item + (*descr).page_size, TRUE);
            }
            SB_PAGELEFT => {
                page = (*descr).width / (*descr).column_width;
                if page < 1 {
                    page = 1;
                }
                LISTBOX_SetTopItem(descr, (*descr).top_item - page * (*descr).page_size, TRUE);
            }
            SB_PAGERIGHT => {
                page = (*descr).width / (*descr).column_width;
                if page < 1 {
                    page = 1;
                }
                LISTBOX_SetTopItem(descr, (*descr).top_item + page * (*descr).page_size, TRUE);
            }
            SB_THUMBPOSITION => {
                LISTBOX_SetTopItem(descr, pos as INT * (*descr).page_size, TRUE);
            }
            SB_THUMBTRACK => {
                info.cbSize = ::core::mem::size_of::<SCROLLINFO>() as UINT;
                info.fMask = SIF_TRACKPOS as UINT;
                GetScrollInfo((*descr).self_, (SB_VERT) as i32, core::ptr::addr_of_mut!(info));
                LISTBOX_SetTopItem(descr, info.nTrackPos as INT * (*descr).page_size, TRUE);
            }
            SB_LEFT => {
                LISTBOX_SetTopItem(descr, 0, TRUE);
            }
            SB_RIGHT => {
                LISTBOX_SetTopItem(descr, (*descr).nb_items, TRUE);
            }
            _ => {}
        }
    } else if (*descr).horz_extent != 0 {
        match (scrollReq as ::core::ffi::c_int) as u32 {
            SB_LINELEFT => {
                LISTBOX_SetHorizontalPos(descr, (*descr).horz_pos - 1);
            }
            SB_LINERIGHT => {
                LISTBOX_SetHorizontalPos(descr, (*descr).horz_pos + 1);
            }
            SB_PAGELEFT => {
                LISTBOX_SetHorizontalPos(descr, (*descr).horz_pos - (*descr).width);
            }
            SB_PAGERIGHT => {
                LISTBOX_SetHorizontalPos(descr, (*descr).horz_pos + (*descr).width);
            }
            SB_THUMBPOSITION => {
                LISTBOX_SetHorizontalPos(descr, pos as INT);
            }
            SB_THUMBTRACK => {
                info.cbSize = ::core::mem::size_of::<SCROLLINFO>() as UINT;
                info.fMask = SIF_TRACKPOS as UINT;
                GetScrollInfo((*descr).self_, (SB_HORZ) as i32, core::ptr::addr_of_mut!(info));
                LISTBOX_SetHorizontalPos(descr, info.nTrackPos as INT);
            }
            SB_LEFT => {
                LISTBOX_SetHorizontalPos(descr, 0);
            }
            SB_RIGHT => {
                LISTBOX_SetHorizontalPos(descr, (*descr).horz_extent - (*descr).width);
            }
            _ => {}
        }
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn LISTBOX_HandleMouseWheel(descr: *mut LB_DESCR, delta: SHORT) -> LRESULT {
    let mut pulScrollLines: INT = 3 as INT;

    SystemParametersInfoW(
        SPI_GETWHEELSCROLLLINES as UINT,
        0 as UINT,
        core::ptr::addr_of_mut!(pulScrollLines) as PVOID,
        0 as UINT,
    );

    /* if scrolling changes direction, ignore left overs */
    if ((delta as INT) < 0 && (*descr).wheel_remain < 0)
        || ((delta as INT) > 0 && (*descr).wheel_remain > 0)
    {
        (*descr).wheel_remain += delta as INT;
    } else {
        (*descr).wheel_remain = delta as INT;
    }

    if (*descr).wheel_remain != 0 && pulScrollLines != 0 {
        let mut cLineScroll: INT;
        if (*descr).style & (LBS_MULTICOLUMN as UINT) != 0 {
            pulScrollLines = min((*descr).width / (*descr).column_width, pulScrollLines);
            pulScrollLines = max(1 as INT, pulScrollLines);
            cLineScroll = pulScrollLines * (*descr).wheel_remain / WHEEL_DELTA as INT;
            (*descr).wheel_remain -= (WHEEL_DELTA * (cLineScroll) as u32 / (pulScrollLines) as u32) as i32;
            cLineScroll *= (*descr).page_size;
        } else {
            pulScrollLines = min((*descr).page_size, pulScrollLines);
            cLineScroll = pulScrollLines * (*descr).wheel_remain / WHEEL_DELTA as INT;
            (*descr).wheel_remain -= (WHEEL_DELTA * (cLineScroll) as u32 / (pulScrollLines) as u32) as i32;
        }
        LISTBOX_SetTopItem(
            descr,
            (*descr).top_item - cLineScroll,
            TRUE as BOOL,
        );
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn LISTBOX_HandleLButtonDown(
    descr: *mut LB_DESCR,
    keys: DWORD,
    x: INT,
    y: INT,
) -> LRESULT {
    let mut index: INT = LISTBOX_GetItemFromPoint(descr, x, y);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listbox.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTBOX_HandleLButtonDown\0" as *const u8 as *const ::core::ffi::c_char,
            2163 as ::core::ffi::c_int,
            b"[%p]: lbuttondown %d,%d item %d, focus item %d\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*descr).self_,
            x,
            y,
            index,
            (*descr).focus_item,
        );
    }

    if (*descr).caret_on == 0 && (*descr).in_focus != 0 {
        return 0 as LRESULT;
    }

    if (*descr).in_focus == 0 {
        if (*descr).lphc.is_null() {
            SetFocus((*descr).self_);
        } else {
            SetFocus(if !(*(*descr).lphc).hWndEdit.is_null() {
                (*(*descr).lphc).hWndEdit
            } else {
                (*(*descr).lphc).self_
            });
        }
    }

    if index == -(1 as INT) {
        return 0 as LRESULT;
    }

    if (*descr).lphc.is_null() {
        if ((*descr).style & (LBS_NOTIFY as UINT)) != 0 {
            SendMessageW(
                (*descr).owner,
                WM_LBTRACKPOINT as UINT,
                index as WPARAM,
                ((x as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as DWORD
                    | ((y as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as DWORD)
                        << 16 as ::core::ffi::c_int) as LONG as DWORD as LPARAM,
            );
        }
    }

    (*descr).captured = TRUE as BOOL;
    SetCapture((*descr).self_);

    if ((*descr).style & ((LBS_EXTENDEDSEL | LBS_MULTIPLESEL) as UINT)) != 0 {
        if (keys & (MK_SHIFT as DWORD)) == 0 {
            (*descr).anchor_item = index;
        }
        if (keys & (MK_CONTROL as DWORD)) != 0 {
            LISTBOX_SetCaretIndex(descr, index, FALSE as BOOL);
            LISTBOX_SetSelection(
                descr,
                index,
                (is_item_selected(descr, index as UINT) == 0) as BOOL,
                (((*descr).style & (LBS_NOTIFY as UINT))
                    != (0 as UINT)) as BOOL,
            );
        } else {
            LISTBOX_MoveCaret(descr, index, FALSE as BOOL);

            if ((*descr).style & (LBS_EXTENDEDSEL as UINT)) != 0 {
                LISTBOX_SetSelection(
                    descr,
                    index,
                    is_item_selected(descr, index as UINT),
                    (((*descr).style & (LBS_NOTIFY as UINT))
                        != (0 as UINT)) as BOOL,
                );
            } else {
                LISTBOX_SetSelection(
                    descr,
                    index,
                    (is_item_selected(descr, index as UINT) == 0) as BOOL,
                    (((*descr).style & (LBS_NOTIFY as UINT))
                        != (0 as UINT)) as BOOL,
                );
            }
        }
    } else {
        (*descr).anchor_item = index;
        LISTBOX_MoveCaret(descr, index, FALSE as BOOL);
        LISTBOX_SetSelection(
            descr,
            index,
            TRUE as BOOL,
            (((*descr).style & (LBS_NOTIFY as UINT))
                != (0 as UINT)) as BOOL,
        );
    }

    if (*descr).lphc.is_null() {
        if (GetWindowLongW((*descr).self_, GWL_EXSTYLE as INT)
            & (WS_EX_DRAGDETECT as INT))
            != 0
        {
            let mut pt: POINT = tagPOINT { x: 0, y: 0 };

            pt.x = x as LONG;
            pt.y = y as LONG;

            if DragDetect((*descr).self_, pt) != 0 {
                SendMessageW(
                    (*descr).owner,
                    WM_BEGINDRAG as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                );
            }
        }
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn LISTBOX_HandleLButtonDownCombo(
    descr: *mut LB_DESCR,
    msg: UINT,
    keys: DWORD,
    x: INT,
    y: INT,
) -> LRESULT {
    let mut clientRect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut screenRect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut mousePos: POINT = POINT { x: 0, y: 0 };

    mousePos.x = x as LONG;
    mousePos.y = y as LONG;

    GetClientRect((*descr).self_, core::ptr::addr_of_mut!(clientRect));

    if PtInRect(core::ptr::addr_of!(clientRect), mousePos) != 0 {
        /* MousePos is in client, resume normal processing */
        if msg == WM_LBUTTONDOWN {
            (*(*descr).lphc).droppedIndex = if (*descr).nb_items != 0 {
                (*descr).selected_item
            } else {
                -1
            };
            return LISTBOX_HandleLButtonDown(descr, keys, x, y);
        } else if ((*descr).style & LBS_NOTIFY) != 0 {
            SEND_NOTIFICATION!(descr, LBN_DBLCLK);
        }
    } else {
        let mut screenMousePos: POINT = POINT { x: 0, y: 0 };
        let mut hWndOldCapture: HWND = core::ptr::null_mut();

        /* Check the Non-Client Area */
        screenMousePos = mousePos;
        hWndOldCapture = GetCapture();
        ReleaseCapture();
        GetWindowRect((*descr).self_, core::ptr::addr_of_mut!(screenRect));
        ClientToScreen((*descr).self_, core::ptr::addr_of_mut!(screenMousePos));

        if PtInRect(core::ptr::addr_of!(screenRect), screenMousePos) == 0 {
            LISTBOX_SetCaretIndex(
                descr,
                (*(*descr).lphc).droppedIndex,
                FALSE,
            );
            LISTBOX_SetSelection(
                descr,
                (*(*descr).lphc).droppedIndex,
                FALSE,
                FALSE,
            );
            COMBO_FlipListbox(
                (*descr).lphc,
                FALSE,
                FALSE,
            );
        } else {
            /* Check to see the NC is a scrollbar */
            let mut nHitTestType: INT = 0;
            let mut style: LONG = GetWindowLongW(
                (*descr).self_,
                GWL_STYLE,
            );
            /* Check Vertical scroll bar */
            if (style as ::core::ffi::c_int & (WS_VSCROLL as ::core::ffi::c_int)) != 0 {
                clientRect.right += GetSystemMetrics((SM_CXVSCROLL) as i32) as LONG;
                if PtInRect(core::ptr::addr_of!(clientRect), mousePos) != 0 {
                    nHitTestType = (HTVSCROLL) as i32;
                }
            }
            /* Check horizontal scroll bar */
            if (style as ::core::ffi::c_int & (WS_HSCROLL as ::core::ffi::c_int)) != 0 {
                clientRect.bottom += GetSystemMetrics((SM_CYHSCROLL) as i32) as LONG;
                if PtInRect(core::ptr::addr_of!(clientRect), mousePos) != 0 {
                    nHitTestType = (HTHSCROLL) as i32;
                }
            }
            /* Windows sends this message when a scrollbar is clicked
             */

            if nHitTestType != 0 {
                SendMessageW(
                    (*descr).self_,
                    WM_NCLBUTTONDOWN,
                    nHitTestType as WPARAM,
                    MAKELONG!(screenMousePos.x, screenMousePos.y) as LPARAM,
                );
            }
            /* Resume the Capture after scrolling is complete
             */
            if hWndOldCapture != core::ptr::null_mut() {
                SetCapture(hWndOldCapture);
            }
        }
    }
    return 0;
}

unsafe extern "C" fn LISTBOX_HandleLButtonUp(mut descr: *mut LB_DESCR) -> LRESULT {
    if LISTBOX_Timer as ::core::ffi::c_uint != (LB_TIMER_NONE as u32) {
        KillSystemTimer((*descr).self_, LB_TIMER_ID as UINT_PTR);
    }
    LISTBOX_Timer = LB_TIMER_NONE;
    if (*descr).captured != 0 {
        (*descr).captured = FALSE;
        if GetCapture() == (*descr).self_ {
            ReleaseCapture();
        }
        if ((*descr).style & LBS_NOTIFY as UINT) != 0 && (*descr).nb_items != 0 {
            SEND_NOTIFICATION!(descr, LBN_SELCHANGE);
        }
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn LISTBOX_HandleTimer(
    mut descr: *mut LB_DESCR,
    mut index: INT,
    mut dir: TIMER_DIRECTION,
) -> LRESULT {
    match (dir as ::core::ffi::c_uint) as i32 {
        LB_TIMER_UP => {
            if (*descr).top_item != 0 {
                index = ((*descr).top_item as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as INT;
            } else {
                index = 0 as ::core::ffi::c_int as INT;
            }
        }
        LB_TIMER_LEFT => {
            if (*descr).top_item != 0 {
                index -= (*descr).page_size;
            }
        }
        LB_TIMER_DOWN => {
            index = (*descr).top_item + LISTBOX_GetCurrentPageSize(descr);
            if index == (*descr).focus_item {
                index += 1;
            }
            if index >= (*descr).nb_items {
                index = ((*descr).nb_items as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as INT;
            }
        }
        LB_TIMER_RIGHT => {
            if index + (*descr).page_size < (*descr).nb_items {
                index += (*descr).page_size;
            }
        }
        LB_TIMER_NONE | _ => {}
    }
    if index == (*descr).focus_item {
        return FALSE as LRESULT;
    }
    LISTBOX_MoveCaret(descr, index, FALSE);
    return TRUE as LRESULT;
}

unsafe extern "C" fn LISTBOX_HandleSystemTimer(descr: *mut LB_DESCR) -> LRESULT {
    if LISTBOX_HandleTimer(descr, (*descr).focus_item, LISTBOX_Timer) == 0 {
        KillSystemTimer((*descr).self_, (LB_TIMER_ID) as UINT_PTR);
        LISTBOX_Timer = LB_TIMER_NONE;
    }
    return 0;
}

unsafe extern "C" fn LISTBOX_HandleMouseMove(mut descr: *mut LB_DESCR, mut x: INT, mut y: INT) {
    let mut index: INT = 0;
    let mut dir: TIMER_DIRECTION = LB_TIMER_NONE;
    if (*descr).captured == 0 {
        return;
    }
    if ((*descr).style & LBS_MULTICOLUMN as UINT) != 0 {
        if y < 0 as INT {
            y = 0 as INT;
        } else if y >= (*descr).item_height * (*descr).page_size {
            y = ((*descr).item_height as ::core::ffi::c_int
                * (*descr).page_size as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as INT;
        }
        if x < 0 as INT {
            dir = LB_TIMER_LEFT;
            x = 0 as INT;
        } else if x >= (*descr).width {
            dir = LB_TIMER_RIGHT;
            x = ((*descr).width as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as INT;
        }
    } else if y < 0 as INT {
        dir = LB_TIMER_UP;
    } else if y >= (*descr).height {
        dir = LB_TIMER_DOWN;
    }
    index = LISTBOX_GetItemFromPoint(descr, x, y);
    if index == -(1 as INT) {
        index = (*descr).focus_item;
    }
    if LISTBOX_HandleTimer(descr, index, dir) == 0 {
        dir = LB_TIMER_NONE;
    }
    if dir as ::core::ffi::c_uint
        != LB_TIMER_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        SetSystemTimer(
            (*descr).self_,
            LB_TIMER_ID as UINT_PTR,
            LB_SCROLL_TIMEOUT as UINT,
            None,
        );
    } else if LISTBOX_Timer as ::core::ffi::c_uint
        != LB_TIMER_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        KillSystemTimer((*descr).self_, LB_TIMER_ID as UINT_PTR);
    }
    LISTBOX_Timer = dir;
}

unsafe extern "C" fn LISTBOX_HandleKeyDown(descr: *mut LB_DESCR, key: DWORD) -> LRESULT {
    let mut caret: INT = -1;
    let mut bForceSelection: BOOL = TRUE; /* select item pointed to by focus_item */
    if (IS_MULTISELECT!(descr) != false) || ((*descr).selected_item == (*descr).focus_item) {
        bForceSelection = FALSE; /* only for single select list */
    }

    if ((*descr).style & LBS_WANTKEYBOARDINPUT) != 0 {
        caret = SendMessageW(
            (*descr).owner,
            WM_VKEYTOITEM,
            MAKEWPARAM!(LOWORD!(key), (*descr).focus_item),
            (*descr).self_ as LPARAM,
        ) as INT;
        if caret == -2 {
            return 0;
        }
    }
    if caret == -1 {
        /* blk: 0 = no fallthrough target, 1 = UP/LEFT body, 2 = DOWN/RIGHT body */
        let mut blk: u8 = 0;
        match key {
            VK_LEFT => {
                if ((*descr).style & LBS_MULTICOLUMN) != 0 {
                    bForceSelection = FALSE;
                    if (*descr).focus_item >= (*descr).page_size {
                        caret = (*descr).focus_item - (*descr).page_size;
                    }
                } else {
                    /* fall through */
                    blk = 1;
                }
            }
            VK_UP => {
                blk = 1;
            }
            VK_RIGHT => {
                if ((*descr).style & LBS_MULTICOLUMN) != 0 {
                    bForceSelection = FALSE;
                    caret = min((*descr).focus_item + (*descr).page_size, (*descr).nb_items - 1);
                } else {
                    /* fall through */
                    blk = 2;
                }
            }
            VK_DOWN => {
                blk = 2;
            }
            VK_PRIOR => {
                if ((*descr).style & LBS_MULTICOLUMN) != 0 {
                    let mut page: INT = (*descr).width / (*descr).column_width;
                    if page < 1 {
                        page = 1;
                    }
                    caret = (*descr).focus_item - (page * (*descr).page_size) + 1;
                } else {
                    caret = (*descr).focus_item - LISTBOX_GetCurrentPageSize(descr) + 1;
                }
                if caret < 0 {
                    caret = 0;
                }
            }
            VK_NEXT => {
                if ((*descr).style & LBS_MULTICOLUMN) != 0 {
                    let mut page: INT = (*descr).width / (*descr).column_width;
                    if page < 1 {
                        page = 1;
                    }
                    caret = (*descr).focus_item + (page * (*descr).page_size) - 1;
                } else {
                    caret = (*descr).focus_item + LISTBOX_GetCurrentPageSize(descr) - 1;
                }
                if caret >= (*descr).nb_items {
                    caret = (*descr).nb_items - 1;
                }
            }
            VK_HOME => {
                caret = 0;
            }
            VK_END => {
                caret = (*descr).nb_items - 1;
            }
            VK_SPACE => {
                if ((*descr).style & LBS_EXTENDEDSEL) != 0 {
                    caret = (*descr).focus_item;
                } else if ((*descr).style & LBS_MULTIPLESEL) != 0 {
                    LISTBOX_SetSelection(
                        descr,
                        (*descr).focus_item,
                        ((is_item_selected(descr, ((*descr).focus_item) as UINT) == 0) as INT) as BOOL,
                        ((((*descr).style & LBS_NOTIFY) != 0) as INT) as BOOL,
                    );
                }
            }
            _ => {
                bForceSelection = FALSE;
            }
        }
        match blk {
            2 => {
                caret = (*descr).focus_item + 1;
                if caret >= (*descr).nb_items {
                    caret = (*descr).nb_items - 1;
                }
            }
            1 => {
                caret = (*descr).focus_item - 1;
                if caret < 0 {
                    caret = 0;
                }
            }
            _ => {}
        }
    }
    if bForceSelection != 0 {
        /* focused item is used instead of key */
        caret = (*descr).focus_item;
    }
    if caret >= 0 {
        if ((((*descr).style & LBS_EXTENDEDSEL) != 0)
            && ((GetKeyState(VK_SHIFT as INT) as i32 & 0x8000) == 0))
            || (IS_MULTISELECT!(descr) == false)
        {
            (*descr).anchor_item = caret;
        }
        LISTBOX_MoveCaret(descr, caret, TRUE);

        if ((*descr).style & LBS_MULTIPLESEL) != 0 {
            (*descr).selected_item = caret;
        } else {
            LISTBOX_SetSelection(descr, caret, TRUE, FALSE);
        }
        if ((*descr).style & LBS_NOTIFY) != 0 {
            if !(*descr).lphc.is_null() && IsWindowVisible((*descr).self_) != 0 {
                /* make sure that combo parent doesn't hide us */
                (*(*descr).lphc).wState |= CBF_NOROLLUP;
            }
            if (*descr).nb_items != 0 {
                SEND_NOTIFICATION!(descr, LBN_SELCHANGE);
            }
        }
    }
    return 0;
}

unsafe extern "C" fn LISTBOX_HandleChar(descr: *mut LB_DESCR, charW: WCHAR) -> LRESULT {
    let mut caret: INT = -(1 as INT);
    let mut str: [WCHAR; 2 as usize] = [0; 2 as usize];

    str[0 as usize] = charW;
    str[1 as usize] = '\0' as WCHAR;

    if (*descr).style & LBS_WANTKEYBOARDINPUT as UINT != 0 {
        caret = SendMessageW(
            (*descr).owner,
            WM_CHARTOITEM as UINT,
            MAKEWPARAM!(charW, (*descr).focus_item) as WPARAM,
            (*descr).self_ as LPARAM,
        ) as INT;
        if caret == -(2 as INT) {
            return 0 as LRESULT;
        }
    }
    if caret == -(1 as INT) {
        caret = LISTBOX_FindString(
            descr,
            (*descr).focus_item,
            core::ptr::addr_of_mut!(str) as *mut WCHAR as LPCWSTR,
            FALSE as BOOL,
        );
    }
    if caret != -(1 as INT) {
        if (!IS_MULTISELECT!(descr)) && (*descr).selected_item == -(1 as INT) {
            LISTBOX_SetSelection(
                descr,
                caret,
                TRUE as BOOL,
                FALSE as BOOL,
            );
        }
        LISTBOX_MoveCaret(descr, caret, TRUE as BOOL);
        if ((*descr).style & LBS_NOTIFY as UINT != 0) && (*descr).nb_items != 0 {
            SEND_NOTIFICATION!(descr, LBN_SELCHANGE);
        }
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn LISTBOX_Create(hwnd: HWND, lphc: LPHEADCOMBO) -> BOOL {
    let mut descr: *mut LB_DESCR = core::ptr::null_mut::<LB_DESCR>();
    let mut mis: MEASUREITEMSTRUCT = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();

    descr = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<LB_DESCR>() as SIZE_T,
    ) as *mut LB_DESCR;
    if descr.is_null() {
        return FALSE;
    }

    GetClientRect(hwnd, core::ptr::addr_of_mut!(rect));
    (*descr).self_ = hwnd;
    (*descr).owner = GetParent((*descr).self_);
    (*descr).style = GetWindowLongW((*descr).self_, GWL_STYLE) as UINT;
    (*descr).width = (rect.right - rect.left) as INT;
    (*descr).height = (rect.bottom - rect.top) as INT;
    (*descr).u.items = core::ptr::null_mut::<LB_ITEMDATA>();
    (*descr).items_size = 0;
    (*descr).nb_items = 0;
    (*descr).top_item = 0;
    (*descr).selected_item = -1;
    (*descr).focus_item = 0;
    (*descr).anchor_item = -1;
    (*descr).item_height = 1;
    (*descr).page_size = 1;
    (*descr).column_width = 150;
    (*descr).horz_extent = 0;
    (*descr).horz_pos = 0;
    (*descr).nb_tabs = 0;
    (*descr).tabs = core::ptr::null_mut::<INT>();
    (*descr).wheel_remain = 0;
    (*descr).caret_on = lphc.is_null() as BOOL;
    if (*descr).style & LBS_NOSEL != 0 {
        (*descr).caret_on = FALSE;
    }
    (*descr).in_focus = FALSE;
    (*descr).captured = FALSE;
    (*descr).font = 0 as HFONT;
    (*descr).locale = GetUserDefaultLCID();
    (*descr).lphc = lphc;

    if !lphc.is_null() {
        (*descr).owner = (*lphc).self_;
    }

    SetWindowLongPtrW((*descr).self_, 0, descr as LONG_PTR);

    if (*descr).style & LBS_EXTENDEDSEL != 0 {
        (*descr).style |= LBS_MULTIPLESEL;
    }
    if (*descr).style & LBS_MULTICOLUMN != 0 {
        (*descr).style &= !LBS_OWNERDRAWVARIABLE;
    }
    if (*descr).style & LBS_OWNERDRAWVARIABLE != 0 {
        (*descr).style |= LBS_NOINTEGRALHEIGHT;
    }
    if ((*descr).style & (LBS_OWNERDRAWFIXED | LBS_HASSTRINGS | LBS_SORT))
        != LBS_OWNERDRAWFIXED
    {
        (*descr).style &= !LBS_NODATA;
    }
    (*descr).item_height = LISTBOX_SetFont(descr, 0 as HFONT);

    if (*descr).style & LBS_OWNERDRAWFIXED != 0 {
        (*descr).style &= !LBS_OWNERDRAWVARIABLE;

        if !(*descr).lphc.is_null() && ((*(*descr).lphc).dwStyle & CBS_DROPDOWN) != 0 {
            /* WinWord gets VERY unhappy if we send WM_MEASUREITEM from here */
            (*descr).item_height = (*lphc).fixedOwnerDrawHeight;
        } else {
            let mut id: UINT = GetWindowLongPtrW((*descr).self_, GWLP_ID) as UINT;
            mis.CtlType = ODT_LISTBOX;
            mis.CtlID = id;
            mis.itemID = (-(1 as i32)) as UINT;
            mis.itemWidth = 0;
            mis.itemData = 0;
            mis.itemHeight = (*descr).item_height as UINT;
            SendMessageW(
                (*descr).owner,
                WM_MEASUREITEM,
                id as WPARAM,
                core::ptr::addr_of_mut!(mis) as LPARAM,
            );
            (*descr).item_height = (if mis.itemHeight != 0 {
                mis.itemHeight
            } else {
                1
            }) as INT;
        }
    }

    OpenThemeData((*descr).self_, (WC_LISTBOXW).as_ptr());

    return TRUE;
}

unsafe extern "C" fn LISTBOX_Destroy(mut descr: *mut LB_DESCR) -> BOOL {
    let mut theme: HTHEME = GetWindowTheme((*descr).self_);
    CloseThemeData(theme);
    LISTBOX_ResetContent(descr);
    SetWindowLongPtrW((*descr).self_, 0, 0);
    HeapFree(GetProcessHeap(), 0, descr as PVOID);
    return TRUE;
}
unsafe extern "system" fn LISTBOX_WindowProc(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut descr: *mut LB_DESCR = GetWindowLongPtrW(hwnd, 0) as *mut LB_DESCR;
    let mut lphc: *mut HEADCOMBO = core::ptr::null_mut();
    let mut theme: HTHEME = core::ptr::null_mut();
    let mut ret: LRESULT = 0;

    if descr.is_null() {
        if IsWindow(hwnd) == 0 {
            return 0 as LRESULT;
        }

        if msg == WM_CREATE {
            let mut lpcs: *mut CREATESTRUCTW = lParam as *mut CREATESTRUCTW;
            if ((*lpcs).style & (LBS_COMBOBOX as LONG)) != 0 {
                lphc = (*lpcs).lpCreateParams as *mut HEADCOMBO;
            }
            if LISTBOX_Create(hwnd, lphc as LPHEADCOMBO) == 0 {
                return -(1) as LRESULT;
            }

            return 0 as LRESULT;
        }
        /* Ignore all other messages before we get a WM_CREATE */
        return DefWindowProcW(hwnd, msg, wParam, lParam);
    }
    if ((*descr).style & (LBS_COMBOBOX as UINT)) != 0 {
        lphc = (*descr).lphc;
    }

    TRACE!(
        b"[%p]: msg %#x wp %08lx lp %08lx\n\0".as_ptr() as *const ::core::ffi::c_char,
        (*descr).self_0,
        msg,
        wParam,
        lParam,
    );

    match msg {
        LB_RESETCONTENT => {
            LISTBOX_ResetContent(descr);
            LISTBOX_UpdateScroll(descr);
            InvalidateRect((*descr).self_, core::ptr::null::<RECT>(), TRUE);
            return 0 as LRESULT;
        }

        LB_ADDSTRING => {
            let textW: *const WCHAR = lParam as *const WCHAR;
            let index: INT = LISTBOX_FindStringPos(descr, textW as LPCWSTR, FALSE);
            return LISTBOX_InsertString(descr, index, textW as LPCWSTR);
        }

        LB_INSERTSTRING => {
            return LISTBOX_InsertString(descr, wParam as INT, lParam as LPCWSTR);
        }

        LB_ADDFILE => {
            let textW: *const WCHAR = lParam as *const WCHAR;
            let index: INT = LISTBOX_FindFileStrPos(descr, textW as LPCWSTR);
            return LISTBOX_InsertString(descr, index, textW as LPCWSTR);
        }

        LB_DELETESTRING => {
            if LISTBOX_RemoveItem(descr, wParam as INT) != (LB_ERR) as _ {
                return (*descr).nb_items as LRESULT;
            } else {
                SetLastError(ERROR_INVALID_INDEX as DWORD);
                return LB_ERR as LRESULT;
            }
        }

        LB_GETITEMDATA => {
            if (wParam as INT) < 0 || (wParam as INT) >= (*descr).nb_items {
                SetLastError(ERROR_INVALID_INDEX as DWORD);
                return LB_ERR as LRESULT;
            }
            return get_item_data(descr, wParam as UINT) as LRESULT;
        }

        LB_SETITEMDATA => {
            if (wParam as INT) < 0 || (wParam as INT) >= (*descr).nb_items {
                SetLastError(ERROR_INVALID_INDEX as DWORD);
                return LB_ERR as LRESULT;
            }
            set_item_data(descr, wParam as UINT, lParam as ULONG_PTR);
            /* undocumented: returns TRUE, not LB_OKAY (0) */
            return TRUE as LRESULT;
        }

        LB_GETCOUNT => {
            return (*descr).nb_items as LRESULT;
        }

        LB_GETTEXT => {
            return LISTBOX_GetText(descr, wParam as INT, lParam as LPWSTR, TRUE);
        }

        LB_GETTEXTLEN => {
            if (wParam as INT) >= (*descr).nb_items || (wParam as INT) < 0 {
                SetLastError(ERROR_INVALID_INDEX as DWORD);
                return LB_ERR as LRESULT;
            }
            if HAS_STRINGS!(descr) == false {
                return core::mem::size_of::<ULONG_PTR>() as LRESULT;
            }
            return lstrlenW(get_item_string(descr, wParam as UINT) as LPCWSTR) as LRESULT;
        }

        LB_GETCURSEL => {
            if (*descr).nb_items == 0 {
                return LB_ERR as LRESULT;
            }
            if IS_MULTISELECT!(descr) == false {
                return (*descr).selected_item as LRESULT;
            }
            if (*descr).selected_item != -1 {
                return (*descr).selected_item as LRESULT;
            }
            return (*descr).focus_item as LRESULT;
            /* otherwise, if the user tries to move the selection with the    */
            /* arrow keys, we will give the application something to choke on */
        }

        LB_GETTOPINDEX => {
            return (*descr).top_item as LRESULT;
        }

        LB_GETITEMHEIGHT => {
            return LISTBOX_GetItemHeight(descr, wParam as INT);
        }

        LB_SETITEMHEIGHT => {
            return LISTBOX_SetItemHeight(descr, wParam as INT, lParam as INT, TRUE);
        }

        LB_ITEMFROMPOINT => {
            let mut pt: POINT = core::mem::zeroed();
            let mut rect: RECT = core::mem::zeroed();
            let mut index: INT = 0;
            let mut hit: BOOL = TRUE;

            /* The hiword of the return value is not a client area
               hittest as suggested by MSDN, but rather a hittest on
               the returned listbox item. */

            if (*descr).nb_items == 0 {
                return 0x1ffff as LRESULT; /* win9x returns 0x10000, we copy winnt */
            }

            pt.x = (LOWORD!(lParam) as SHORT) as LONG;
            pt.y = (HIWORD!(lParam) as SHORT) as LONG;

            SetRect(
                core::ptr::addr_of_mut!(rect),
                0,
                0,
                (*descr).width,
                (*descr).height,
            );

            if PtInRect(core::ptr::addr_of!(rect), pt) == 0 {
                pt.x = min(pt.x, rect.right - 1 as LONG);
                pt.x = max(pt.x, 0 as LONG);
                pt.y = min(pt.y, rect.bottom - 1 as LONG);
                pt.y = max(pt.y, 0 as LONG);
                hit = FALSE;
            }

            index = LISTBOX_GetItemFromPoint(descr, pt.x as INT, pt.y as INT);

            if index == -1 {
                index = (*descr).nb_items - 1;
                hit = FALSE;
            }
            return MAKELONG!(index, if hit != 0 { 0 } else { 1 }) as LRESULT;
        }

        LB_SETCARETINDEX => {
            if IS_MULTISELECT!(descr) == false && (*descr).selected_item != -1 {
                return LB_ERR as LRESULT;
            }
            if LISTBOX_SetCaretIndex(descr, wParam as INT, (lParam == 0) as BOOL) == (LB_ERR) as _ {
                return LB_ERR as LRESULT;
            } else if LOWORD!(GetVersion()) == 0x0a03 {
                return wParam as LRESULT;
            } else {
                return LB_OKAY as LRESULT;
            }
        }

        LB_GETCARETINDEX => {
            return (*descr).focus_item as LRESULT;
        }

        LB_SETTOPINDEX => {
            return LISTBOX_SetTopItem(descr, wParam as INT, TRUE);
        }

        LB_SETCOLUMNWIDTH => {
            return LISTBOX_SetColumnWidth(descr, wParam as INT);
        }

        LB_GETITEMRECT => {
            return LISTBOX_GetItemRect(descr, wParam as INT, lParam as *mut RECT);
        }

        LB_FINDSTRING => {
            return (LISTBOX_FindString(descr, wParam as INT, lParam as LPCWSTR, FALSE)) as LRESULT;
        }

        LB_FINDSTRINGEXACT => {
            return (LISTBOX_FindString(descr, wParam as INT, lParam as LPCWSTR, TRUE)) as LRESULT;
        }

        LB_SELECTSTRING => {
            let textW: *const WCHAR = lParam as *const WCHAR;

            if HAS_STRINGS!(descr) != false {
                TRACE!(
                    b"LB_SELECTSTRING: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                    debugstr_w(textW),
                );
            }

            let index: INT = LISTBOX_FindString(descr, wParam as INT, textW as LPCWSTR, FALSE);
            if index != LB_ERR {
                LISTBOX_MoveCaret(descr, index, TRUE);
                LISTBOX_SetSelection(descr, index, TRUE, FALSE);
            }
            return index as LRESULT;
        }

        LB_GETSEL => {
            if (wParam as INT) < 0 || (wParam as INT) >= (*descr).nb_items {
                return LB_ERR as LRESULT;
            }
            return is_item_selected(descr, wParam as UINT) as LRESULT;
        }

        LB_SETSEL => {
            ret = LISTBOX_SetSelection(descr, lParam as INT, wParam as BOOL, FALSE);
            if ret != (LB_ERR) as _ && wParam != 0 {
                (*descr).anchor_item = lParam as INT;
                if lParam != -(1) as LPARAM {
                    LISTBOX_SetCaretIndex(descr, lParam as INT, TRUE);
                }
            }
            return ret;
        }

        LB_SETCURSEL => {
            if IS_MULTISELECT!(descr) != false {
                return LB_ERR as LRESULT;
            }
            LISTBOX_SetCaretIndex(descr, wParam as INT, TRUE);
            ret = LISTBOX_SetSelection(descr, wParam as INT, TRUE, FALSE);
            if ret != (LB_ERR) as _ {
                ret = (*descr).selected_item as LRESULT;
            }
            return ret;
        }

        LB_GETSELCOUNT => {
            return LISTBOX_GetSelCount(descr);
        }

        LB_GETSELITEMS => {
            return LISTBOX_GetSelItems(descr, wParam as INT, lParam as LPINT);
        }

        LB_SELITEMRANGE => {
            if (LOWORD!(lParam) as INT) <= (HIWORD!(lParam) as INT) {
                return LISTBOX_SelectItemRange(
                    descr,
                    LOWORD!(lParam) as INT,
                    HIWORD!(lParam) as INT,
                    wParam as BOOL,
                );
            } else {
                return LISTBOX_SelectItemRange(
                    descr,
                    HIWORD!(lParam) as INT,
                    LOWORD!(lParam) as INT,
                    wParam as BOOL,
                );
            }
        }

        LB_SELITEMRANGEEX => {
            if (lParam as INT) >= (wParam as INT) {
                return LISTBOX_SelectItemRange(descr, wParam as INT, lParam as INT, TRUE);
            } else {
                return LISTBOX_SelectItemRange(descr, lParam as INT, wParam as INT, FALSE);
            }
        }

        LB_GETHORIZONTALEXTENT => {
            return (*descr).horz_extent as LRESULT;
        }

        LB_SETHORIZONTALEXTENT => {
            return LISTBOX_SetHorizontalExtent(descr, wParam as INT);
        }

        LB_GETANCHORINDEX => {
            return (*descr).anchor_item as LRESULT;
        }

        LB_SETANCHORINDEX => {
            if (wParam as INT) < -1 || (wParam as INT) >= (*descr).nb_items {
                SetLastError(ERROR_INVALID_INDEX as DWORD);
                return LB_ERR as LRESULT;
            }
            (*descr).anchor_item = wParam as INT;
            return LB_OKAY as LRESULT;
        }

        LB_DIR => {
            return LISTBOX_Directory(
                descr,
                wParam as UINT,
                lParam as LPCWSTR,
                (msg == LB_DIR) as BOOL,
            ) as LRESULT;
        }

        LB_GETLOCALE => {
            return (*descr).locale as LRESULT;
        }

        LB_SETLOCALE => {
            let mut ret: LCID = 0;
            if IsValidLocale(wParam as LCID, LCID_INSTALLED) == 0 {
                return LB_ERR as LRESULT;
            }
            ret = (*descr).locale;
            (*descr).locale = wParam as LCID;
            return ret as LRESULT;
        }

        LB_INITSTORAGE => {
            return LISTBOX_InitStorage(descr, wParam as INT);
        }

        LB_SETCOUNT => {
            return LISTBOX_SetCount(descr, wParam as INT as UINT);
        }

        LB_SETTABSTOPS => {
            return LISTBOX_SetTabStops(descr, wParam as INT, lParam as LPINT) as LRESULT;
        }

        LB_CARETON => {
            if (*descr).caret_on != 0 {
                return LB_OKAY as LRESULT;
            }
            (*descr).caret_on = TRUE as BOOL;
            if (*descr).focus_item != -1 && (*descr).in_focus != 0 {
                LISTBOX_RepaintItem(descr, (*descr).focus_item, ODA_FOCUS);
            }
            return LB_OKAY as LRESULT;
        }

        LB_CARETOFF => {
            if (*descr).caret_on == 0 {
                return LB_OKAY as LRESULT;
            }
            (*descr).caret_on = FALSE as BOOL;
            if (*descr).focus_item != -1 && (*descr).in_focus != 0 {
                LISTBOX_RepaintItem(descr, (*descr).focus_item, ODA_FOCUS);
            }
            return LB_OKAY as LRESULT;
        }

        LB_GETLISTBOXINFO => {
            return (*descr).page_size as LRESULT;
        }

        WM_DESTROY => {
            return (LISTBOX_Destroy(descr)) as LRESULT;
        }

        WM_ENABLE => {
            InvalidateRect((*descr).self_, core::ptr::null::<RECT>(), TRUE);
            return 0 as LRESULT;
        }

        WM_SETREDRAW => {
            LISTBOX_SetRedraw(descr, (wParam != 0) as BOOL);
            return 0 as LRESULT;
        }

        WM_GETDLGCODE => {
            return (DLGC_WANTARROWS | DLGC_WANTCHARS) as LRESULT;
        }

        WM_PRINTCLIENT | WM_PAINT => {
            let mut ps: PAINTSTRUCT = core::mem::zeroed();
            let hdc: HDC = if wParam != 0 {
                wParam as HDC
            } else {
                BeginPaint((*descr).self_, core::ptr::addr_of_mut!(ps))
            };
            ret = LISTBOX_Paint(descr, hdc);
            if wParam == 0 {
                EndPaint((*descr).self_, core::ptr::addr_of_mut!(ps));
            }
            return ret;
        }

        WM_NCPAINT => {
            LISTBOX_NCPaint(descr, wParam as HRGN);
        }

        WM_SIZE => {
            LISTBOX_UpdateSize(descr);
            return 0 as LRESULT;
        }

        WM_GETFONT => {
            return (*descr).font as LRESULT;
        }

        WM_SETFONT => {
            LISTBOX_SetFont(descr, wParam as HFONT);
            if lParam != 0 {
                InvalidateRect(
                    (*descr).self_,
                    (0 as usize as *const RECT),
                    TRUE,
                );
            }
            return 0 as LRESULT;
        }

        WM_SETFOCUS => {
            (*descr).in_focus = TRUE as BOOL;
            (*descr).caret_on = TRUE as BOOL;
            if (*descr).focus_item != -1 {
                LISTBOX_DrawFocusRect(descr, TRUE);
            }
            SEND_NOTIFICATION!(descr, LBN_SETFOCUS);
            return 0 as LRESULT;
        }

        WM_KILLFOCUS => {
            LISTBOX_HandleLButtonUp(descr); /* Release capture if we have it */
            (*descr).in_focus = FALSE as BOOL;
            (*descr).wheel_remain = 0 as INT;
            if (*descr).focus_item != -1 && (*descr).caret_on != 0 {
                LISTBOX_RepaintItem(descr, (*descr).focus_item, ODA_FOCUS);
            }
            SEND_NOTIFICATION!(descr, LBN_KILLFOCUS);
            return 0 as LRESULT;
        }

        WM_HSCROLL => {
            return LISTBOX_HandleHScroll(descr, LOWORD!(wParam) as WORD, HIWORD!(wParam) as WORD);
        }

        WM_VSCROLL => {
            return LISTBOX_HandleVScroll(descr, LOWORD!(wParam) as WORD, HIWORD!(wParam) as WORD);
        }

        WM_MOUSEWHEEL => {
            if (wParam & ((MK_SHIFT | MK_CONTROL) as WPARAM)) != 0 {
                return DefWindowProcW((*descr).self_, msg, wParam, lParam);
            }
            return LISTBOX_HandleMouseWheel(descr, HIWORD!(wParam) as SHORT);
        }

        WM_LBUTTONDOWN => {
            if !lphc.is_null() {
                return LISTBOX_HandleLButtonDownCombo(
                    descr,
                    msg,
                    wParam as DWORD,
                    (LOWORD!(lParam) as INT16) as INT,
                    (HIWORD!(lParam) as INT16) as INT,
                );
            }
            return LISTBOX_HandleLButtonDown(
                descr,
                wParam as DWORD,
                (LOWORD!(lParam) as INT16) as INT,
                (HIWORD!(lParam) as INT16) as INT,
            );
        }

        WM_LBUTTONDBLCLK => {
            if !lphc.is_null() {
                return LISTBOX_HandleLButtonDownCombo(
                    descr,
                    msg,
                    wParam as DWORD,
                    (LOWORD!(lParam) as INT16) as INT,
                    (HIWORD!(lParam) as INT16) as INT,
                );
            }
            if ((*descr).style & (LBS_NOTIFY as UINT)) != 0 {
                SEND_NOTIFICATION!(descr, LBN_DBLCLK);
            }
            return 0 as LRESULT;
        }

        WM_MOUSEMOVE => {
            if !lphc.is_null()
                && ((*lphc).dwStyle & (CBS_DROPDOWNLIST as DWORD)) != (CBS_SIMPLE as DWORD)
            {
                let captured: BOOL = (*descr).captured;
                let mut mousePos: POINT = core::mem::zeroed();
                let mut clientRect: RECT = core::mem::zeroed();

                mousePos.x = (LOWORD!(lParam) as INT16) as LONG;
                mousePos.y = (HIWORD!(lParam) as INT16) as LONG;

                /*
                 * If we are in a dropdown combobox, we simulate that
                 * the mouse is captured to show the tracking of the item.
                 */
                if GetClientRect((*descr).self_, core::ptr::addr_of_mut!(clientRect)) != 0
                    && PtInRect(core::ptr::addr_of!(clientRect), mousePos) != 0
                {
                    (*descr).captured = TRUE as BOOL;
                }

                LISTBOX_HandleMouseMove(descr, mousePos.x as INT, mousePos.y as INT);

                (*descr).captured = captured;
            } else if GetCapture() == (*descr).self_ {
                LISTBOX_HandleMouseMove(
                    descr,
                    (LOWORD!(lParam) as INT16) as INT,
                    (HIWORD!(lParam) as INT16) as INT,
                );
            }
            return 0 as LRESULT;
        }

        WM_LBUTTONUP => {
            if !lphc.is_null() {
                let mut mousePos: POINT = core::mem::zeroed();
                let mut clientRect: RECT = core::mem::zeroed();

                /*
                 * If the mouse button "up" is not in the listbox,
                 * we make sure there is no selection by re-selecting the
                 * item that was selected when the listbox was made visible.
                 */
                mousePos.x = (LOWORD!(lParam) as INT16) as LONG;
                mousePos.y = (HIWORD!(lParam) as INT16) as LONG;

                GetClientRect((*descr).self_, core::ptr::addr_of_mut!(clientRect));

                /*
                 * When the user clicks outside the combobox and the focus
                 * is lost, the owning combobox will send a fake buttonup with
                 * 0xFFFFFFF as the mouse location, we must also revert the
                 * selection to the original selection.
                 */
                if (lParam == (-(1)) as LPARAM)
                    || (PtInRect(core::ptr::addr_of!(clientRect), mousePos) == 0)
                {
                    LISTBOX_MoveCaret(descr, (*lphc).droppedIndex, FALSE);
                }
            }
            return LISTBOX_HandleLButtonUp(descr);
        }

        WM_KEYDOWN => {
            if !lphc.is_null()
                && ((*lphc).dwStyle & (CBS_DROPDOWNLIST as DWORD)) != (CBS_SIMPLE as DWORD)
            {
                /* for some reason Windows makes it possible to
                 * show/hide ComboLBox by sending it WM_KEYDOWNs */

                if (((*lphc).wState & (CBF_EUI as UINT)) == 0 && wParam == (VK_F4) as _)
                    || (((*lphc).wState & (CBF_EUI as UINT)) != 0
                        && ((*lphc).wState & (CBF_DROPPED as UINT)) == 0
                        && (wParam == (VK_DOWN) as _ || wParam == (VK_UP) as _))
                {
                    COMBO_FlipListbox(lphc, FALSE, FALSE);
                    return 0 as LRESULT;
                }
            }
            return LISTBOX_HandleKeyDown(descr, wParam as DWORD);
        }

        WM_CHAR => {
            return LISTBOX_HandleChar(descr, wParam as WCHAR);
        }

        WM_SYSTIMER => {
            return LISTBOX_HandleSystemTimer(descr);
        }

        WM_ERASEBKGND => {
            if (IS_OWNERDRAW!(descr) != false) && ((*descr).style & (LBS_DISPLAYCHANGED as UINT)) == 0 {
                let mut rect: RECT = core::mem::zeroed();
                let mut hbrush: HBRUSH = SendMessageW(
                    (*descr).owner,
                    WM_CTLCOLORLISTBOX,
                    wParam,
                    (*descr).self_ as LPARAM,
                ) as HBRUSH;

                if hbrush.is_null() {
                    hbrush = GetSysColorBrush((COLOR_WINDOW) as i32);
                }
                if !hbrush.is_null() {
                    GetClientRect((*descr).self_, core::ptr::addr_of_mut!(rect));
                    FillRect(wParam as HDC, core::ptr::addr_of!(rect), hbrush);
                }
            }
            return 1 as LRESULT;
        }

        WM_DROPFILES => {
            if !lphc.is_null() {
                return 0 as LRESULT;
            }
            return SendMessageW((*descr).owner, msg, wParam, lParam);
        }

        WM_NCDESTROY => {
            if !lphc.is_null() && ((*lphc).dwStyle & (CBS_DROPDOWNLIST as DWORD)) != (CBS_SIMPLE as DWORD) {
                (*lphc).hWndLBox = 0 as usize as HWND;
            }
        }

        WM_NCACTIVATE => {
            if !lphc.is_null() {
                return 0 as LRESULT;
            }
        }

        WM_THEMECHANGED => {
            theme = GetWindowTheme(hwnd);
            CloseThemeData(theme);
            OpenThemeData(hwnd, (WC_LISTBOXW).as_ptr());
        }

        _ => {
            if msg >= WM_USER && msg < 0xc000 {
                WARN!(
                    b"[%p]: unknown msg %04x wp %08lx lp %08lx\n\0".as_ptr() as *const ::core::ffi::c_char,
                    hwnd,
                    msg,
                    wParam,
                    lParam,
                );
            }
        }
    }

    return DefWindowProcW(hwnd, msg, wParam, lParam);
}
#[no_mangle]
pub unsafe extern "C" fn LISTBOX_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    memset(
        &raw mut wndClass as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<WNDCLASSW>() as size_t,
    );
    wndClass.style = (CS_PARENTDC | CS_DBLCLKS | CS_GLOBALCLASS) as UINT;
    wndClass.lpfnWndProc = Some(LISTBOX_WindowProc);
    wndClass.cbClsExtra = 0 as ::core::ffi::c_int;
    wndClass.cbWndExtra = ::core::mem::size_of::<*mut LB_DESCR>() as ::core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        0 as HINSTANCE,
        IDC_ARROW as LPWSTR as LPCWSTR,
    );
    wndClass.hbrBackground = ::core::ptr::null_mut::<HBRUSH__>();
    wndClass.lpszClassName = WC_LISTBOXW.as_ptr() as LPCWSTR;
    RegisterClassW(&raw mut wndClass);
}
#[no_mangle]
pub unsafe extern "C" fn COMBOLBOX_Register() {
    static mut combolboxW: [WCHAR; 10] = [0x0043u16, 0x006F, 0x006D, 0x0062, 0x006F, 0x004C, 0x0042, 0x006F, 0x0078, 0x0000];
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    memset(core::ptr::addr_of_mut!(wndClass) as *mut core::ffi::c_void, 0 as core::ffi::c_int, core::mem::size_of::<WNDCLASSW>() as size_t);
    wndClass.style = (CS_SAVEBITS | CS_DBLCLKS | CS_DROPSHADOW | CS_GLOBALCLASS) as UINT;
    wndClass.lpfnWndProc = Some(LISTBOX_WindowProc);
    wndClass.cbClsExtra = 0 as core::ffi::c_int;
    wndClass.cbWndExtra = core::mem::size_of::<*mut LB_DESCR>() as core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(0 as HINSTANCE, (IDC_ARROW as LPWSTR));
    wndClass.hbrBackground = core::ptr::null_mut();
    wndClass.lpszClassName = core::ptr::addr_of!(combolboxW) as *const WCHAR as LPCWSTR;
    RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn LISTBOX_Unregister() {
    UnregisterClassW((WC_LISTBOXW).as_ptr(), core::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "C" fn COMBOLBOX_Unregister() {
    static combolboxW: [u16; 10] = [0x0043u16, 0x006F, 0x006D, 0x0062, 0x006F, 0x004C, 0x0042, 0x006F, 0x0078, 0x0000];
    UnregisterClassW(core::ptr::addr_of!(combolboxW) as *const u16, core::ptr::null_mut());
}
