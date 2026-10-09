use crate::comctl32::commctrl::COMCTL32_CreateToolTip;
use crate::comctl32::commctrl::COMCTL32_IsReflectedMessage;
use crate::comctl32::commctrl::COMCTL32_RefreshSysColors;
use crate::comctl32::dpa::DPA_Create;
use crate::comctl32::dpa::DPA_DeletePtr;
use crate::comctl32::dpa::DPA_Destroy;
use crate::comctl32::dpa::DPA_EnumCallback;
use crate::comctl32::dpa::DPA_GetPtr;
use crate::comctl32::dpa::DPA_GetPtrIndex;
use crate::comctl32::dpa::DPA_InsertPtr;
use crate::comctl32::dpa::DPA_Sort;
use crate::comctl32::imagelist::ImageList_Add;
use crate::comctl32::imagelist::ImageList_AddMasked;
use crate::comctl32::imagelist::ImageList_Create;
use crate::comctl32::imagelist::ImageList_Draw;
use crate::comctl32::imagelist::ImageList_DrawEx;
use crate::comctl32::imagelist::ImageList_GetIconSize;
use crate::comctl32::commctrl::_TrackMouseEvent;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_treeview);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const TOOLTIPS_CLASSW: [::core::ffi::c_ushort; 17] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_ushort; 17]>(
        *b"t\0o\0o\0l\0t\0i\0p\0s\0_\0c\0l\0a\0s\0s\x003\x002\0\0\0",
    )
};

pub const WC_TREEVIEWW: [::core::ffi::c_ushort; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_ushort; 14]>(
        *b"S\0y\0s\0T\0r\0e\0e\0V\0i\0e\0w\x003\x002\0\0\0",
    )
};

pub const WC_EDITW: [::core::ffi::c_ushort; 5] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_ushort; 5]>(*b"E\0d\0i\0t\0\0\0") };

static mut themeClass: [WCHAR; 9] = [
    'T' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

// NONAMELESSUNION: empty object-like macro, no declaration emitted.
// WINE_DEFAULT_DEBUG_CHANNEL(treeview): expands to the debug channel object for this file, handled by the debug shim.

#[repr(C)]
pub struct TREEVIEW_INFO {
    pub hwnd: HWND,
    pub hwndNotify: HWND,
    pub dwStyle: DWORD,
    pub root: HTREEITEM,
    pub uInternalStatus: UINT,
    pub Timer: INT,
    pub uNumItems: UINT,
    pub cdmode: INT,
    pub uScrollTime: UINT,
    pub bRedraw: BOOL,
    pub uItemHeight: UINT,
    pub bHeightSet: BOOL,
    pub clientWidth: LONG,
    pub clientHeight: LONG,
    pub treeWidth: LONG,
    pub treeHeight: LONG,
    pub uIndent: UINT,
    pub selectedItem: HTREEITEM,
    pub hotItem: HTREEITEM,
    pub focusedItem: HTREEITEM,
    pub editItem: HTREEITEM,
    pub firstVisible: HTREEITEM,
    pub maxVisibleOrder: LONG,
    pub dropItem: HTREEITEM,
    pub insertMarkItem: HTREEITEM,
    pub insertBeforeorAfter: BOOL,
    pub dragList: HIMAGELIST,
    pub scrollX: LONG,
    pub wheelRemainder: INT,
    pub clrBk: COLORREF,
    pub clrText: COLORREF,
    pub clrLine: COLORREF,
    pub clrInsertMark: COLORREF,
    pub hFont: HFONT,
    pub hDefaultFont: HFONT,
    pub hBoldFont: HFONT,
    pub hUnderlineFont: HFONT,
    pub hBoldUnderlineFont: HFONT,
    pub hcurHand: HCURSOR,
    pub hwndToolTip: HWND,
    pub hwndEdit: HWND,
    pub wpEditOrig: WNDPROC,
    pub bIgnoreEditKillFocus: BOOL,
    pub bLabelChanged: BOOL,
    pub bNtfUnicode: BOOL,
    pub himlNormal: HIMAGELIST,
    pub normalImageHeight: core::ffi::c_int,
    pub normalImageWidth: core::ffi::c_int,
    pub himlState: HIMAGELIST,
    pub stateImageHeight: core::ffi::c_int,
    pub stateImageWidth: core::ffi::c_int,
    pub items: HDPA,
    pub lastKeyPressTimestamp: DWORD,
    pub charCode: WPARAM,
    pub nSearchParamLength: INT,
    pub szSearchParam: [WCHAR; (MAX_PATH) as _],
}

#[repr(C)]
pub struct TREEVIEW_ITEM {
    pub parent: HTREEITEM,
    pub nextSibling: HTREEITEM,
    pub firstChild: HTREEITEM,
    pub callbackMask: UINT,
    pub state: UINT,
    pub stateMask: UINT,
    pub pszText: LPWSTR,
    pub cchTextMax: core::ffi::c_int,
    pub iImage: core::ffi::c_int,
    pub iSelectedImage: core::ffi::c_int,
    pub iExpandedImage: core::ffi::c_int,
    pub cChildren: core::ffi::c_int,
    pub lParam: LPARAM,
    pub iIntegral: core::ffi::c_int,
    pub iLevel: core::ffi::c_int,
    pub lastChild: HTREEITEM,
    pub prevSibling: HTREEITEM,
    pub rect: RECT,
    pub linesOffset: LONG,
    pub stateOffset: LONG,
    pub imageOffset: LONG,
    pub textOffset: LONG,
    pub textWidth: LONG,
    pub visibleOrder: LONG,
    pub infoPtr: *const TREEVIEW_INFO,
}

pub const KEY_DELAY: i32 = 450;

pub const TV_HSCROLL: i32 = 0x01;

pub const TV_VSCROLL: i32 = 0x02;

pub const TV_LDRAG: i32 = 0x04;

pub const TV_LDRAGGING: i32 = 0x08;

pub const TV_RDRAG: i32 = 0x10;

pub const TV_RDRAGGING: i32 = 0x20;

pub const TV_EDIT_TIMER: i32 = 2;

pub const TV_EDIT_TIMER_SET: i32 = 2;

pub const TEXT_CALLBACK_SIZE: i32 = 260;

pub const TREEVIEW_LEFT_MARGIN: i32 = 8;

pub const MINIMUM_INDENT: i32 = 19;

pub const CALLBACK_MASK_ALL: u32 = TVIF_TEXT | TVIF_CHILDREN | TVIF_IMAGE | TVIF_SELECTEDIMAGE;

// STATEIMAGEINDEX, OVERLAYIMAGEINDEX, ISVISIBLE, GETLINECOLOR, GETBKCOLOR,
// GETTXTCOLOR, GETINSCOLOR: function-like macros, skipped.

pub type TREEVIEW_ItemEnumFunc =
    unsafe extern "C" fn(*mut TREEVIEW_INFO, *mut TREEVIEW_ITEM, LPVOID);

// Static function prototypes (TREEVIEW_Invalidate, TREEVIEW_DoSelectItem,
// TREEVIEW_SetFirstVisible, TREEVIEW_EnsureVisible, TREEVIEW_EndEditLabelNow,
// TREEVIEW_UpdateScrollBars, TREEVIEW_HScroll, TREEVIEW_VerifyTree,
// TREEVIEW_RemoveItem, TREEVIEW_SelectItem, TREEVIEW_VerifyChildren): function
// declarations only, skipped; the caller assembles the function definitions.

pub const TVHEIGHT_MIN: i32 = 16;

pub const TVHEIGHT_FONT_ADJUST: i32 = 3;

unsafe extern "C" fn TREEVIEW_GetInfoPtr(hwnd: HWND) -> *mut TREEVIEW_INFO {
    return GetWindowLongPtrW(hwnd, 0) as isize as *mut TREEVIEW_INFO;
}

unsafe extern "C" fn TREEVIEW_GetItemIndex(infoPtr: *const TREEVIEW_INFO, handle: HTREEITEM) -> i32 {
    return DPA_GetPtrIndex((*infoPtr).items, handle as *const core::ffi::c_void);
}

unsafe extern "C" fn item_changed(
    mut tiOld: *const TREEVIEW_ITEM,
    mut tiNew: *const TREEVIEW_ITEM,
    mut tvChange: *const TVITEMEXW,
) -> BOOL {
    /* Number of children has changed */
    if ((*tvChange).mask & (TVIF_CHILDREN as UINT)) != 0 && (*tiOld).cChildren != (*tiNew).cChildren {
        return TRUE;
    }

    /* Image has changed and it's not a callback */
    if ((*tvChange).mask & (TVIF_IMAGE as UINT)) != 0
        && (*tiOld).iImage != (*tiNew).iImage
        && (*tiNew).iImage != I_IMAGECALLBACK
    {
        return TRUE;
    }

    /* Selected image has changed and it's not a callback */
    if ((*tvChange).mask & (TVIF_SELECTEDIMAGE as UINT)) != 0
        && (*tiOld).iSelectedImage != (*tiNew).iSelectedImage
        && (*tiNew).iSelectedImage != I_IMAGECALLBACK
    {
        return TRUE;
    }

    if ((*tvChange).mask & (TVIF_EXPANDEDIMAGE as UINT)) != 0
        && (*tiOld).iExpandedImage != (*tiNew).iExpandedImage
        && (*tiNew).iExpandedImage != I_IMAGECALLBACK
    {
        return TRUE;
    }

    /* Text has changed and it's not a callback */
    if ((*tvChange).mask & (TVIF_TEXT as UINT)) != 0
        && (*tiOld).pszText != (*tiNew).pszText
        && (*tiNew).pszText != LPSTR_TEXTCALLBACKW
    {
        return TRUE;
    }

    /* Indent has changed */
    if ((*tvChange).mask & (TVIF_INTEGRAL as UINT)) != 0 && (*tiOld).iIntegral != (*tiNew).iIntegral {
        return TRUE;
    }

    /* Item state has changed */
    if ((*tvChange).mask & (TVIF_STATE as UINT)) != 0
        && (((*tiOld).state ^ (*tiNew).state) & (*tvChange).stateMask) != 0
    {
        return TRUE;
    }

    return FALSE;
}

unsafe extern "C" fn TREEVIEW_ValidItem(infoPtr: *const TREEVIEW_INFO, handle: HTREEITEM) -> BOOL {
    if TREEVIEW_GetItemIndex(infoPtr, handle) == -1 {
        return FALSE;
    } else {
        return TRUE;
    }
}

unsafe extern "C" fn TREEVIEW_CreateBoldFont(hOrigFont: HFONT) -> HFONT {
    let mut font: LOGFONTW = core::mem::zeroed();
    GetObjectW(hOrigFont as HANDLE, core::mem::size_of::<LOGFONTW>() as core::ffi::c_int, core::ptr::addr_of_mut!(font) as LPVOID);
    font.lfWeight = FW_BOLD as LONG;
    return CreateFontIndirectW(core::ptr::addr_of_mut!(font));
}

static TREEVIEW_CreateUnderlineFont: unsafe extern "C" fn(HFONT) -> HFONT = {
    unsafe extern "C" fn TREEVIEW_CreateUnderlineFont(hOrigFont: HFONT) -> HFONT {
        let mut font: LOGFONTW = core::mem::zeroed();

        GetObjectW(
            hOrigFont as HANDLE,
            core::mem::size_of::<LOGFONTW>() as i32,
            core::ptr::addr_of_mut!(font) as LPVOID,
        );
        font.lfUnderline = TRUE as BYTE;
        return CreateFontIndirectW(core::ptr::addr_of_mut!(font));
    }
    TREEVIEW_CreateUnderlineFont
};

unsafe extern "C" fn TREEVIEW_CreateBoldUnderlineFont(hfont: HFONT) -> HFONT {
    let mut font: LOGFONTW = core::mem::zeroed();

    GetObjectW(hfont as HANDLE, core::mem::size_of::<LOGFONTW>() as i32, core::ptr::addr_of_mut!(font) as LPVOID);
    font.lfWeight = FW_BOLD as LONG;
    font.lfUnderline = TRUE as BYTE;
    return CreateFontIndirectW(core::ptr::addr_of_mut!(font));
}

unsafe extern "C" fn TREEVIEW_FontForItem(
    mut infoPtr: *const TREEVIEW_INFO,
    mut item: *const TREEVIEW_ITEM,
) -> HFONT {
    if ((*infoPtr).dwStyle & (TVS_TRACKSELECT as DWORD) != 0)
        && item == (*infoPtr).hotItem as *const TREEVIEW_ITEM
    {
        return if (*item).state & (TVIS_BOLD as UINT) != 0 {
            (*infoPtr).hBoldUnderlineFont
        } else {
            (*infoPtr).hUnderlineFont
        };
    }
    if (*item).state & (TVIS_BOLD as UINT) != 0 {
        return (*infoPtr).hBoldFont;
    }
    return (*infoPtr).hFont;
}

unsafe extern "C" fn TREEVIEW_ItemName(
    item: *const TREEVIEW_ITEM,
) -> *const ::core::ffi::c_char {
    if item.is_null() {
        return b"<null item>\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*item).pszText == LPSTR_TEXTCALLBACKW {
        return b"<callback>\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*item).pszText.is_null() {
        return b"<null>\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return debugstr_w((*item).pszText as *const WCHAR);
}

unsafe extern "C" fn TREEVIEW_IsChildOf(
    parent: *const TREEVIEW_ITEM,
    mut child: *const TREEVIEW_ITEM,
) -> BOOL {
    loop {
        child = (*child).parent as *const TREEVIEW_ITEM;
        if child == parent {
            return TRUE;
        }
        if child.is_null() {
            break;
        }
    }
    return FALSE;
}

unsafe extern "C" fn TREEVIEW_IsFullRowSelect(infoPtr: *const TREEVIEW_INFO) -> BOOL {
    return (!((*infoPtr).dwStyle & TVS_HASLINES != 0)
        && ((*infoPtr).dwStyle & TVS_FULLROWSELECT != 0)) as BOOL;
}

unsafe extern "C" fn TREEVIEW_IsItemHit(infoPtr: *const TREEVIEW_INFO, ht: *const TVHITTESTINFO) -> BOOL {
    if TREEVIEW_IsFullRowSelect(infoPtr) != 0 {
        return ((*ht).flags & ((TVHT_ONITEMINDENT | TVHT_ONITEMBUTTON | (TVHT_ONITEM as u32) | TVHT_ONITEMRIGHT) as UINT)) as BOOL;
    } else {
        return ((*ht).flags & (TVHT_ONITEM as UINT)) as BOOL;
    }
}

unsafe extern "C" fn TREEVIEW_GetLastListItem(infoPtr: *const TREEVIEW_INFO, mut item: *mut TREEVIEW_ITEM) -> *mut TREEVIEW_ITEM {
    if item.is_null() {
        return core::ptr::null_mut();
    }

    while !(*item).lastChild.is_null() {
        if ((*item).state & (TVIS_EXPANDED as UINT)) != 0 {
            item = (*item).lastChild as *mut TREEVIEW_ITEM;
        } else {
            break;
        }
    }

    if item == ((*infoPtr).root).cast() {
        return core::ptr::null_mut();
    }

    return item;
}

unsafe extern "C" fn TREEVIEW_GetPrevListItem(
    infoPtr: *const TREEVIEW_INFO,
    tvItem: *const TREEVIEW_ITEM,
) -> *mut TREEVIEW_ITEM {
    if !(*tvItem).prevSibling.is_null() {
        /* This item has a prevSibling, get the last item in the sibling's tree. */
        let mut upItem: *mut TREEVIEW_ITEM = (*tvItem).prevSibling as *mut TREEVIEW_ITEM;

        if ((*upItem).state & (TVIS_EXPANDED as UINT)) != 0 && !(*upItem).lastChild.is_null() {
            return TREEVIEW_GetLastListItem(infoPtr, (*upItem).lastChild as *mut TREEVIEW_ITEM);
        } else {
            return upItem;
        }
    } else {
        /* this item does not have a prevSibling, get the parent */
        return if (*tvItem).parent != (*infoPtr).root {
            (*tvItem).parent as *mut TREEVIEW_ITEM
        } else {
            core::ptr::null_mut::<TREEVIEW_ITEM>()
        };
    }
}

unsafe extern "C" fn TREEVIEW_GetNextListItem(
    infoPtr: *const TREEVIEW_INFO,
    mut tvItem: *const TREEVIEW_ITEM,
) -> *mut TREEVIEW_ITEM {
    if ((*tvItem).state & TVIS_EXPANDED) != 0 && !(*tvItem).firstChild.is_null() {
        return (*tvItem).firstChild as *mut TREEVIEW_ITEM;
    }

    if !(*tvItem).nextSibling.is_null() {
        return (*tvItem).nextSibling as *mut TREEVIEW_ITEM;
    }

    while !(*tvItem).parent.is_null() {
        tvItem = (*tvItem).parent as *const TREEVIEW_ITEM;

        if !(*tvItem).nextSibling.is_null() {
            return (*tvItem).nextSibling as *mut TREEVIEW_ITEM;
        }
    }

    return core::ptr::null_mut();
}

unsafe extern "C" fn TREEVIEW_GetListItem(
    infoPtr: *const TREEVIEW_INFO,
    mut item: *mut TREEVIEW_ITEM,
    mut count: LONG,
) -> *mut TREEVIEW_ITEM {
    let next_item: unsafe extern "C" fn(*const TREEVIEW_INFO, *const TREEVIEW_ITEM) -> *mut TREEVIEW_ITEM;
    let mut previousItem: *mut TREEVIEW_ITEM;

    assert!((!item.is_null()) as i32);

    if count > 0 {
        next_item = TREEVIEW_GetNextListItem;
    } else if count < 0 {
        count = -count;
        next_item = TREEVIEW_GetPrevListItem;
    } else {
        return item;
    }

    loop {
        previousItem = item;
        item = next_item(infoPtr, item);
        count -= 1;
        if !(count != 0 && !item.is_null()) {
            break;
        }
    }

    return if !item.is_null() { item } else { previousItem };
}

unsafe extern "C" fn get_notifycode(infoPtr: *const TREEVIEW_INFO, code: INT) -> INT {
    if (*infoPtr).bNtfUnicode == 0 {
        match code {
            TVN_SELCHANGINGW => return TVN_SELCHANGINGA as INT,
            TVN_SELCHANGEDW => return TVN_SELCHANGEDA as INT,
            TVN_GETDISPINFOW => return TVN_GETDISPINFOA as INT,
            TVN_SETDISPINFOW => return TVN_SETDISPINFOA as INT,
            TVN_ITEMEXPANDINGW => return TVN_ITEMEXPANDINGA as INT,
            TVN_ITEMEXPANDEDW => return TVN_ITEMEXPANDEDA as INT,
            TVN_BEGINDRAGW => return TVN_BEGINDRAGA as INT,
            TVN_BEGINRDRAGW => return TVN_BEGINRDRAGA as INT,
            TVN_DELETEITEMW => return TVN_DELETEITEMA as INT,
            TVN_BEGINLABELEDITW => return TVN_BEGINLABELEDITA as INT,
            TVN_ENDLABELEDITW => return TVN_ENDLABELEDITA as INT,
            TVN_GETINFOTIPW => return TVN_GETINFOTIPA as INT,
            _ => {}
        }
    }
    return code;
}

unsafe extern "C" fn TREEVIEW_SendRealNotify(infoPtr: *const TREEVIEW_INFO, code: UINT, hdr: *mut NMHDR) -> BOOL {
    (*hdr).hwndFrom = (*infoPtr).hwnd;
    (*hdr).idFrom = GetWindowLongPtrW((*infoPtr).hwnd, GWLP_ID) as UINT_PTR;
    (*hdr).code = get_notifycode(infoPtr, code as INT) as UINT;

    return SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFY as UINT,
        (*hdr).idFrom as WPARAM,
        hdr as LPARAM,
    ) as BOOL;
}

unsafe extern "C" fn TREEVIEW_SendSimpleNotify(infoPtr: *const TREEVIEW_INFO, code: UINT) -> BOOL {
    let mut hdr: NMHDR = core::mem::zeroed();
    return TREEVIEW_SendRealNotify(infoPtr, code, core::ptr::addr_of_mut!(hdr));
}

unsafe extern "C" fn TREEVIEW_TVItemFromItem(infoPtr: *const TREEVIEW_INFO, mask: UINT, tvItem: *mut TVITEMW, item: *mut TREEVIEW_ITEM) {
    (*tvItem).mask = mask;
    (*tvItem).hItem = (item as HTREEITEM).cast();
    (*tvItem).state = (*item).state;
    (*tvItem).stateMask = 0;
    (*tvItem).iImage = (*item).iImage;
    (*tvItem).iSelectedImage = (*item).iSelectedImage;
    (*tvItem).cChildren = (*item).cChildren;
    (*tvItem).lParam = (*item).lParam;

    if (mask & TVIF_TEXT) != 0 {
        if (*infoPtr).bNtfUnicode == 0 {
            (*tvItem).cchTextMax = WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                (*item).pszText as LPCWSTR,
                -1,
                0 as LPSTR,
                0,
                0 as LPCSTR,
                0 as LPBOOL,
            );
            (*tvItem).pszText = heap_alloc((*tvItem).cchTextMax as SIZE_T) as LPWSTR;
            WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                (*item).pszText as LPCWSTR,
                -1,
                (*tvItem).pszText as LPSTR,
                (*tvItem).cchTextMax,
                0 as LPCSTR,
                0 as LPBOOL,
            );
        } else {
            (*tvItem).cchTextMax = (*item).cchTextMax;
            (*tvItem).pszText = (*item).pszText;
        }
    } else {
        (*tvItem).cchTextMax = 0;
        (*tvItem).pszText = core::ptr::null_mut();
    }
}

unsafe extern "C" fn TREEVIEW_SendTreeviewNotify(
    infoPtr: *const TREEVIEW_INFO,
    code: UINT,
    action: UINT,
    mask: UINT,
    oldItem: HTREEITEM,
    newItem: HTREEITEM,
) -> BOOL {
    let mut nmhdr: NMTREEVIEWW = core::mem::zeroed();
    let mut ret: BOOL = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TREEVIEW_SendTreeviewNotify\0" as *const u8 as *const ::core::ffi::c_char,
            576 as ::core::ffi::c_int,
            b"code:%d action:0x%x olditem:%p newitem:%p\n\0" as *const u8
                as *const ::core::ffi::c_char,
            code,
            action,
            oldItem,
            newItem,
        );
    }
    memset(
        &raw mut nmhdr as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        core::mem::size_of::<NMTREEVIEWW>() as usize,
    );
    nmhdr.action = action;

    if !oldItem.is_null() {
        TREEVIEW_TVItemFromItem(
            infoPtr,
            mask,
            &raw mut nmhdr.itemOld,
            oldItem as *mut TREEVIEW_ITEM,
        );
    }

    if !newItem.is_null() {
        TREEVIEW_TVItemFromItem(
            infoPtr,
            mask,
            &raw mut nmhdr.itemNew,
            newItem as *mut TREEVIEW_ITEM,
        );
    }

    nmhdr.ptDrag.x = 0 as LONG;
    nmhdr.ptDrag.y = 0 as LONG;

    ret = TREEVIEW_SendRealNotify(infoPtr, code, &raw mut nmhdr.hdr);
    if (*infoPtr).bNtfUnicode == 0 {
        heap_free(nmhdr.itemOld.pszText as *mut ::core::ffi::c_void);
        heap_free(nmhdr.itemNew.pszText as *mut ::core::ffi::c_void);
    }
    return ret;
}

unsafe extern "C" fn TREEVIEW_SendTreeviewDnDNotify(
    infoPtr: *const TREEVIEW_INFO,
    code: UINT,
    dragItem: HTREEITEM,
    pt: POINT,
) -> BOOL {
    let mut nmhdr: NMTREEVIEWW = core::mem::zeroed::<NMTREEVIEWW>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TREEVIEW_SendTreeviewDnDNotify\0" as *const u8
                as *const ::core::ffi::c_char,
            605 as ::core::ffi::c_int,
            b"code:%d dragitem:%p\n\0" as *const u8 as *const ::core::ffi::c_char,
            code,
            dragItem,
        );
    }
    nmhdr.action = 0 as UINT;
    nmhdr.itemNew.mask = (TVIF_STATE | TVIF_PARAM | TVIF_HANDLE) as UINT;
    nmhdr.itemNew.hItem = (dragItem).cast();
    nmhdr.itemNew.state = (*dragItem).state;
    nmhdr.itemNew.lParam = (*dragItem).lParam;
    nmhdr.ptDrag.x = pt.x;
    nmhdr.ptDrag.y = pt.y;
    return TREEVIEW_SendRealNotify(infoPtr, code, core::ptr::addr_of_mut!(nmhdr.hdr));
}

unsafe extern "C" fn TREEVIEW_SendCustomDrawNotify(
    infoPtr: *const TREEVIEW_INFO,
    dwDrawStage: DWORD,
    hdc: HDC,
    rc: RECT,
) -> BOOL {
    let mut nmcdhdr: NMTVCUSTOMDRAW = core::mem::zeroed();
    let mut nmcd: *mut NMCUSTOMDRAW = core::ptr::null_mut();

    nmcd = core::ptr::addr_of_mut!(nmcdhdr.nmcd);
    (*nmcd).dwDrawStage = dwDrawStage;
    (*nmcd).hdc = hdc;
    (*nmcd).rc = rc;
    (*nmcd).dwItemSpec = 0;
    (*nmcd).uItemState = 0;
    (*nmcd).lItemlParam = 0;
    nmcdhdr.clrText = (*infoPtr).clrText;
    nmcdhdr.clrTextBk = (*infoPtr).clrBk;
    nmcdhdr.iLevel = 0;

    return TREEVIEW_SendRealNotify(infoPtr, NM_CUSTOMDRAW, core::ptr::addr_of_mut!(nmcdhdr.nmcd.hdr));
}

unsafe extern "C" fn TREEVIEW_SendCustomDrawItemNotify(
    infoPtr: *const TREEVIEW_INFO,
    hdc: HDC,
    item: *mut TREEVIEW_ITEM,
    uItemDrawState: UINT,
    nmcdhdr: *mut NMTVCUSTOMDRAW,
) -> BOOL {
    let mut nmcd: *mut NMCUSTOMDRAW = core::ptr::null_mut::<NMCUSTOMDRAW>();
    let mut dwDrawStage: DWORD = 0;
    let mut dwItemSpec: DWORD_PTR = 0;
    let mut uItemState: UINT = 0;

    dwDrawStage = (CDDS_ITEM as UINT | uItemDrawState) as DWORD;
    dwItemSpec = item as DWORD_PTR;
    uItemState = 0 as UINT;
    if (*item).state & TVIS_SELECTED as UINT != 0 {
        uItemState |= CDIS_SELECTED as UINT;
    }
    if item == ((*infoPtr).selectedItem).cast() {
        uItemState |= CDIS_FOCUS as UINT;
    }
    if item == ((*infoPtr).hotItem).cast() {
        uItemState |= CDIS_HOT as UINT;
    }

    nmcd = core::ptr::addr_of_mut!((*nmcdhdr).nmcd);
    (*nmcd).dwDrawStage = dwDrawStage;
    (*nmcd).hdc = hdc;
    (*nmcd).rc = (*item).rect;
    (*nmcd).dwItemSpec = dwItemSpec;
    (*nmcd).uItemState = uItemState;
    (*nmcd).lItemlParam = (*item).lParam;
    (*nmcdhdr).iLevel = (*item).iLevel;

    return TREEVIEW_SendRealNotify(
        infoPtr,
        NM_CUSTOMDRAW as UINT,
        core::ptr::addr_of_mut!((*nmcdhdr).nmcd.hdr),
    );
}

unsafe extern "C" fn TREEVIEW_BeginLabelEditNotify(
    infoPtr: *const TREEVIEW_INFO,
    editItem: *mut TREEVIEW_ITEM,
) -> BOOL {
    let mut tvdi: NMTVDISPINFOW = tagTVDISPINFOW {
        hdr: tagNMHDR {
            hwndFrom: core::ptr::null_mut::<HWND__>(),
            idFrom: 0,
            code: 0,
        },
        item: tagTVITEMW {
            mask: 0,
            hItem: (core::ptr::null_mut::<_TREEITEM>()).cast(),
            state: 0,
            stateMask: 0,
            pszText: core::ptr::null_mut::<WCHAR>(),
            cchTextMax: 0,
            iImage: 0,
            iSelectedImage: 0,
            cChildren: 0,
            lParam: 0,
        },
    };
    let mut ret: BOOL = 0;
    TREEVIEW_TVItemFromItem(
        infoPtr,
        (TVIF_HANDLE | TVIF_STATE | TVIF_PARAM | TVIF_TEXT) as UINT,
        core::ptr::addr_of_mut!(tvdi.item),
        editItem,
    );
    ret = TREEVIEW_SendRealNotify(
        infoPtr,
        TVN_BEGINLABELEDITW as UINT,
        core::ptr::addr_of_mut!(tvdi.hdr),
    );
    if (*infoPtr).bNtfUnicode == 0 {
        heap_free(tvdi.item.pszText as *mut ::core::ffi::c_void);
    }
    return ret;
}

unsafe extern "C" fn TREEVIEW_UpdateDispInfo(infoPtr: *const TREEVIEW_INFO, item: *mut TREEVIEW_ITEM, mut mask: UINT) {
    let mut callback: NMTVDISPINFOEXW = core::mem::zeroed();

    mask &= (*item).callbackMask;

    if mask == 0 {
        return;
    }

    callback.item.pszText = (*item).pszText;
    callback.item.cchTextMax = (*item).cchTextMax;
    callback.item.mask = mask;
    callback.item.hItem = (item as HTREEITEM).cast();
    callback.item.state = (*item).state;
    callback.item.lParam = (*item).lParam;

    if (mask & TVIF_TEXT) != 0 {
        (*item).textWidth = 0 as LONG;
    }

    TREEVIEW_SendRealNotify(infoPtr, TVN_GETDISPINFOW as UINT, core::ptr::addr_of_mut!(callback.hdr));

    mask &= (*item).callbackMask;

    if (mask & TVIF_TEXT) != 0 && callback.item.pszText != (*item).pszText {
        if (*infoPtr).bNtfUnicode == 0 && callback.hdr.code == TVN_GETDISPINFOA as UINT {
            let mut newText: LPWSTR = core::ptr::null_mut::<WCHAR>();
            let mut buflen: ::core::ffi::c_int = 0;
            let mut len: ::core::ffi::c_int = MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                callback.item.pszText as LPSTR as LPCSTR,
                -1,
                core::ptr::null_mut::<WCHAR>(),
                0,
            );
            buflen = max(
                (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
                TEXT_CALLBACK_SIZE as usize,
            ) as ::core::ffi::c_int;
            newText = heap_realloc((*item).pszText as LPVOID, buflen as SIZE_T) as LPWSTR;

            if !newText.is_null() {
                (*item).pszText = newText;
                MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    callback.item.pszText as LPSTR as LPCSTR,
                    -1,
                    (*item).pszText,
                    (buflen as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
                );
                (*item).cchTextMax =
                    (buflen as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int;
            }
        } else {
            let mut len: ::core::ffi::c_int = max(
                lstrlenW(callback.item.pszText as LPCWSTR) + 1,
                TEXT_CALLBACK_SIZE,
            );
            let mut newText: LPWSTR = heap_realloc(
                (*item).pszText as LPVOID,
                (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as SIZE_T,
            ) as LPWSTR;

            if !newText.is_null() {
                (*item).pszText = newText;
                lstrcpyW((*item).pszText, callback.item.pszText as LPCWSTR);
                (*item).cchTextMax = len;
            }
        }
    } else if (mask & TVIF_TEXT) != 0 {
        if (*infoPtr).bNtfUnicode == 0 && callback.hdr.code == TVN_GETDISPINFOA as UINT {
            let mut newText: LPWSTR = core::ptr::null_mut::<WCHAR>();
            let mut buflen: ::core::ffi::c_int = 0;
            let mut len: ::core::ffi::c_int = MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                callback.item.pszText as LPSTR as LPCSTR,
                -1,
                core::ptr::null_mut::<WCHAR>(),
                0,
            );
            buflen = max(
                (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
                TEXT_CALLBACK_SIZE as usize,
            ) as ::core::ffi::c_int;
            newText = heap_alloc(buflen as SIZE_T) as LPWSTR;

            if !newText.is_null() {
                let mut oldText: LPWSTR = (*item).pszText;
                (*item).pszText = newText;
                MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    callback.item.pszText as LPSTR as LPCSTR,
                    -1,
                    (*item).pszText,
                    (buflen as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
                );
                (*item).cchTextMax =
                    (buflen as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int;
                heap_free(oldText as LPVOID);
            }
        }
    }

    if (mask & TVIF_IMAGE) != 0 {
        (*item).iImage = callback.item.iImage;
    }

    if (mask & TVIF_SELECTEDIMAGE) != 0 {
        (*item).iSelectedImage = callback.item.iSelectedImage;
    }

    if (mask & TVIF_EXPANDEDIMAGE) != 0 {
        (*item).iExpandedImage = callback.item.iExpandedImage;
    }

    if (mask & TVIF_CHILDREN) != 0 {
        (*item).cChildren = callback.item.cChildren;
    }

    if (callback.item.mask & TVIF_STATE) != 0 {
        (*item).state &= !callback.item.stateMask;
        (*item).state |= callback.item.state & callback.item.stateMask;
    }

    if (callback.item.mask & TVIF_DI_SETITEM) != 0 {
        (*item).callbackMask &= !callback.item.mask;
    }
}

unsafe extern "C" fn TREEVIEW_HasChildren(infoPtr: *const TREEVIEW_INFO, item: *mut TREEVIEW_ITEM) -> BOOL {
    TREEVIEW_UpdateDispInfo(infoPtr, item, TVIF_CHILDREN);
    /* Protect for a case when callback field is not changed by a host,
       otherwise negative values trigger normal notifications. */
    return ((*item).cChildren != 0 && (*item).cChildren != I_CHILDRENCALLBACK) as BOOL;
}

unsafe extern "C" fn TREEVIEW_NotifyFormat(
    infoPtr: *mut TREEVIEW_INFO,
    hwndFrom: HWND,
    nCommand: UINT,
) -> INT {
    let mut format: INT = 0;

    if nCommand != NF_REQUERY as UINT {
        return 0 as INT;
    }

    format = SendMessageW(
        hwndFrom,
        WM_NOTIFYFORMAT as UINT,
        (*infoPtr).hwnd as WPARAM,
        NF_QUERY as LPARAM,
    ) as INT;

    /* Invalid format returned by NF_QUERY defaults to ANSI*/
    if format != NFR_ANSI as INT && format != NFR_UNICODE as INT {
        format = NFR_ANSI as INT;
    }

    (*infoPtr).bNtfUnicode = (format == NFR_UNICODE as INT) as BOOL;

    return format;
}

unsafe extern "C" fn TREEVIEW_ComputeItemInternalMetrics(infoPtr: *const TREEVIEW_INFO, item: *mut TREEVIEW_ITEM) {
    let lar: BOOL = (((*infoPtr).dwStyle & (TVS_LINESATROOT as DWORD | TVS_HASLINES as DWORD | TVS_HASBUTTONS as DWORD)) > TVS_LINESATROOT as DWORD) as BOOL;
    (*item).linesOffset = (*infoPtr)
        .uIndent
        .wrapping_mul(
            (if lar != 0 {
                (*item).iLevel
            } else {
                (*item).iLevel - 1 as INT
            }) as UINT,
        )
        .wrapping_sub((*infoPtr).scrollX as UINT) as LONG;
    (*item).stateOffset = ((*item).linesOffset as UINT).wrapping_add((*infoPtr).uIndent) as LONG;
    (*item).imageOffset = ((*item).stateOffset as INT
        + (if STATEIMAGEINDEX!((*item).state) != 0 {
            (*infoPtr).stateImageWidth
        } else {
            0 as INT
        })) as LONG;
    (*item).textOffset = ((*item).imageOffset as INT + (*infoPtr).normalImageWidth) as LONG;
}

unsafe extern "C" fn TREEVIEW_ComputeTextWidth(infoPtr: *const TREEVIEW_INFO, item: *mut TREEVIEW_ITEM, hDC: HDC) {
    let mut hdc: HDC = core::ptr::null_mut();
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut sz: SIZE = core::mem::zeroed();

    /* DRAW's OM docker creates items like this */
    if (*item).pszText.is_null() {
        (*item).textWidth = 0;
        return;
    }

    if hDC != core::ptr::null_mut() {
        hdc = hDC;
    } else {
        hdc = GetDC((*infoPtr).hwnd);
        hOldFont = SelectObject(hdc, TREEVIEW_FontForItem(infoPtr, item) as HGDIOBJ) as HFONT;
    }

    GetTextExtentPoint32W(
        hdc,
        (*item).pszText as LPCWSTR,
        lstrlenW((*item).pszText as LPCWSTR),
        &raw mut sz,
    );
    (*item).textWidth = sz.cx;

    if hDC == core::ptr::null_mut() {
        SelectObject(hdc, hOldFont as HGDIOBJ);
        ReleaseDC(core::ptr::null_mut(), hdc);
    }
}

unsafe extern "C" fn TREEVIEW_ComputeItemRect(infoPtr: *const TREEVIEW_INFO, item: *mut TREEVIEW_ITEM) {
    (*item).rect.top = (*infoPtr)
        .uItemHeight
        .wrapping_mul(((*item).visibleOrder - (*(*infoPtr).firstVisible).visibleOrder) as UINT)
        as LONG;

    (*item).rect.bottom = ((*item).rect.top as UINT)
        .wrapping_add((*infoPtr).uItemHeight.wrapping_mul((*item).iIntegral as UINT))
        .wrapping_sub(1 as UINT) as LONG;

    (*item).rect.left = 0 as LONG;
    (*item).rect.right = (*infoPtr).clientWidth;
}

unsafe extern "C" fn TREEVIEW_RecalculateVisibleOrder(
    mut infoPtr: *mut TREEVIEW_INFO,
    mut start: *mut TREEVIEW_ITEM,
) {
    let mut item: *mut TREEVIEW_ITEM = core::ptr::null_mut::<TREEVIEW_ITEM>();
    let mut order: ::core::ffi::c_int = 0;

    if start.is_null() {
        start = (*(*infoPtr).root).firstChild as *mut TREEVIEW_ITEM;
        order = 0;
    } else {
        order = (*start).visibleOrder as ::core::ffi::c_int;
    }

    item = start;
    while !item.is_null() {
        if !ISVISIBLE!(item) && order > 0 {
            TREEVIEW_ComputeItemInternalMetrics(infoPtr, item);
        }
        (*item).visibleOrder = order as LONG;
        order += (*item).iIntegral;
        item = TREEVIEW_GetNextListItem(infoPtr, item);
    }

    (*infoPtr).maxVisibleOrder = order as LONG;

    item = (*(*infoPtr).root).firstChild as *mut TREEVIEW_ITEM;
    while !item.is_null() {
        TREEVIEW_ComputeItemRect(infoPtr, item);
        item = TREEVIEW_GetNextListItem(infoPtr, item);
    }
}

unsafe extern "C" fn TREEVIEW_UpdateSubTree(infoPtr: *const TREEVIEW_INFO, mut root: *mut TREEVIEW_ITEM) {
    let mut sibling: *mut TREEVIEW_ITEM = core::ptr::null_mut();
    let mut hdc: HDC = core::ptr::null_mut();
    let mut hOldFont: HFONT = core::ptr::null_mut();

    if (*root).firstChild.is_null() || ((*root).state & (TVIS_EXPANDED as UINT)) == 0 {
        return;
    }

    (*root).state &= !(TVIS_EXPANDED as UINT);
    sibling = TREEVIEW_GetNextListItem(infoPtr, root);
    (*root).state |= TVIS_EXPANDED as UINT;

    hdc = GetDC((*infoPtr).hwnd);
    hOldFont = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;

    while root != sibling {
        TREEVIEW_ComputeItemInternalMetrics(infoPtr, root);

        if ((*root).callbackMask & (TVIF_TEXT as UINT)) != 0 {
            TREEVIEW_UpdateDispInfo(infoPtr, root, TVIF_TEXT as UINT);
        }

        if (*root).textWidth == 0 as LONG {
            SelectObject(hdc, TREEVIEW_FontForItem(infoPtr, root) as HGDIOBJ);
            TREEVIEW_ComputeTextWidth(infoPtr, root, hdc);
        }

        root = TREEVIEW_GetNextListItem(infoPtr, root);
    }

    SelectObject(hdc, hOldFont as HGDIOBJ);
    ReleaseDC((*infoPtr).hwnd, hdc);
}

unsafe extern "C" fn TREEVIEW_AllocateItem(infoPtr: *const TREEVIEW_INFO) -> *mut TREEVIEW_ITEM {
    let mut newItem: *mut TREEVIEW_ITEM = heap_alloc_zero(
        core::mem::size_of::<TREEVIEW_ITEM>() as SIZE_T,
    ) as *mut TREEVIEW_ITEM;

    if newItem.is_null() {
        return core::ptr::null_mut::<TREEVIEW_ITEM>();
    }

    // I_IMAGENONE would make more sense but this is neither what is
    // documented (MSDN doesn't specify) nor what Windows actually does
    // (it sets it to zero)... and I can so imagine an application using
    // inc/dec to toggle the images.
    (*newItem).iImage = 0 as ::core::ffi::c_int;
    (*newItem).iSelectedImage = 0 as ::core::ffi::c_int;
    (*newItem).iExpandedImage = I_IMAGENONE as WORD as ::core::ffi::c_int;
    (*newItem).infoPtr = infoPtr;

    if DPA_InsertPtr(
        (*infoPtr).items,
        (INT_MAX) as INT,
        newItem as *mut ::core::ffi::c_void,
    ) == -(1 as ::core::ffi::c_int)
    {
        heap_free(newItem as *mut ::core::ffi::c_void);
        return core::ptr::null_mut::<TREEVIEW_ITEM>();
    }

    return newItem;
}

unsafe extern "C" fn TREEVIEW_FreeItem(infoPtr: *mut TREEVIEW_INFO, item: *mut TREEVIEW_ITEM) {
    DPA_DeletePtr((*infoPtr).items, DPA_GetPtrIndex((*infoPtr).items, item as *const ::core::ffi::c_void));
    if (*infoPtr).selectedItem == (item).cast() {
        (*infoPtr).selectedItem = core::ptr::null_mut();
    }
    if (*infoPtr).hotItem == (item).cast() {
        (*infoPtr).hotItem = core::ptr::null_mut();
    }
    if (*infoPtr).focusedItem == (item).cast() {
        (*infoPtr).focusedItem = core::ptr::null_mut();
    }
    if (*infoPtr).firstVisible == (item).cast() {
        (*infoPtr).firstVisible = core::ptr::null_mut();
    }
    if (*infoPtr).dropItem == (item).cast() {
        (*infoPtr).dropItem = core::ptr::null_mut();
    }
    if (*infoPtr).insertMarkItem == (item).cast() {
        (*infoPtr).insertMarkItem = core::ptr::null_mut();
    }
    heap_free(item as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn TREEVIEW_InsertBefore(
    newItem: *mut TREEVIEW_ITEM,
    sibling: *mut TREEVIEW_ITEM,
    parent: *mut TREEVIEW_ITEM,
) {
    assert!(!parent.is_null());

    if !sibling.is_null() {
        assert!((*sibling).parent == (parent).cast());

        if !(*sibling).prevSibling.is_null() {
            (*(*sibling).prevSibling).nextSibling = newItem as HTREEITEM;
        }

        (*newItem).prevSibling = (*sibling).prevSibling;
        (*sibling).prevSibling = newItem as HTREEITEM;
    } else {
        (*newItem).prevSibling = core::ptr::null_mut();
    }

    (*newItem).nextSibling = sibling as HTREEITEM;

    if (*parent).firstChild == sibling as HTREEITEM {
        (*parent).firstChild = newItem as HTREEITEM;
    }

    if (*parent).lastChild.is_null() {
        (*parent).lastChild = newItem as HTREEITEM;
    }
}

unsafe extern "C" fn TREEVIEW_InsertAfter(
    newItem: *mut TREEVIEW_ITEM,
    sibling: *mut TREEVIEW_ITEM,
    parent: *mut TREEVIEW_ITEM,
) {
    assert!(!parent.is_null());

    if !sibling.is_null() {
        assert!((*sibling).parent == (parent).cast());

        if !(*sibling).nextSibling.is_null() {
            (*(*sibling).nextSibling).prevSibling = newItem as HTREEITEM;
        }

        (*newItem).nextSibling = (*sibling).nextSibling;
        (*sibling).nextSibling = newItem as HTREEITEM;
    } else {
        (*newItem).nextSibling = core::ptr::null_mut::<_TREEITEM>();
    }

    (*newItem).prevSibling = sibling as HTREEITEM;

    if (*parent).lastChild == (sibling).cast() {
        (*parent).lastChild = newItem as HTREEITEM;
    }

    if (*parent).firstChild.is_null() {
        (*parent).firstChild = newItem as HTREEITEM;
    }
}

unsafe extern "C" fn TREEVIEW_DoSetItemT(
    infoPtr: *const TREEVIEW_INFO,
    item: *mut TREEVIEW_ITEM,
    tvItem: *const TVITEMEXW,
    isW: BOOL,
) -> BOOL {
    let mut callbackClear: UINT = 0;
    let mut callbackSet: UINT = 0;

    /* Do this first in case it fails. */
    if ((*tvItem).mask & TVIF_TEXT) != 0 {
        (*item).textWidth = 0; /* force width recalculation */

        /* Covers != TEXTCALLBACKA too, and undocumented: pszText of NULL also means TEXTCALLBACK */
        if (*tvItem).pszText != LPSTR_TEXTCALLBACKW && !(*tvItem).pszText.is_null() {
            let mut len: i32 = 0;
            let mut newText: LPWSTR = core::ptr::null_mut::<WCHAR>();
            if isW != 0 {
                len = lstrlenW((*tvItem).pszText as LPCWSTR) + 1;
            } else {
                len = MultiByteToWideChar(
                    CP_ACP,
                    0,
                    (*tvItem).pszText as LPSTR as LPCSTR,
                    -1,
                    core::ptr::null_mut::<WCHAR>(),
                    0,
                );
            }

            /* Allocate new block to make pointer comparison in item_changed() work. */
            newText = heap_alloc(
                (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as LPWSTR;

            if newText.is_null() {
                return FALSE;
            }

            callbackClear |= TVIF_TEXT;

            heap_free((*item).pszText as *mut core::ffi::c_void);
            (*item).pszText = newText;
            (*item).cchTextMax = len;
            if isW != 0 {
                lstrcpynW((*item).pszText, (*tvItem).pszText as LPCWSTR, len);
            } else {
                MultiByteToWideChar(
                    CP_ACP,
                    0,
                    (*tvItem).pszText as LPSTR as LPCSTR,
                    -1,
                    (*item).pszText,
                    len,
                );
            }
        } else {
            callbackSet |= TVIF_TEXT;
            (*item).pszText = heap_realloc(
                (*item).pszText as *mut core::ffi::c_void,
                (TEXT_CALLBACK_SIZE as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as LPWSTR;
            (*item).cchTextMax = TEXT_CALLBACK_SIZE;
        }
    }

    if ((*tvItem).mask & TVIF_CHILDREN) != 0 {
        (*item).cChildren = (*tvItem).cChildren;

        if (*item).cChildren == I_CHILDRENCALLBACK {
            callbackSet |= TVIF_CHILDREN;
        } else {
            callbackClear |= TVIF_CHILDREN;
        }
    }

    if ((*tvItem).mask & TVIF_IMAGE) != 0 {
        (*item).iImage = (*tvItem).iImage;

        if (*item).iImage == I_IMAGECALLBACK {
            callbackSet |= TVIF_IMAGE;
        } else {
            callbackClear |= TVIF_IMAGE;
        }
    }

    if ((*tvItem).mask & TVIF_SELECTEDIMAGE) != 0 {
        (*item).iSelectedImage = (*tvItem).iSelectedImage;

        if (*item).iSelectedImage == I_IMAGECALLBACK {
            callbackSet |= TVIF_SELECTEDIMAGE;
        } else {
            callbackClear |= TVIF_SELECTEDIMAGE;
        }
    }

    if ((*tvItem).mask & TVIF_EXPANDEDIMAGE) != 0 {
        (*item).iExpandedImage = (*tvItem).iExpandedImage;

        if (*item).iExpandedImage == I_IMAGECALLBACK {
            callbackSet |= TVIF_EXPANDEDIMAGE;
        } else {
            callbackClear |= TVIF_EXPANDEDIMAGE;
        }
    }

    if ((*tvItem).mask & TVIF_PARAM) != 0 {
        (*item).lParam = (*tvItem).lParam;
    }

    /* If the application sets TVIF_INTEGRAL without
     * supplying a TVITEMEX structure, it's toast. */
    if ((*tvItem).mask & TVIF_INTEGRAL) != 0 {
        (*item).iIntegral = (*tvItem).iIntegral;
    }

    if ((*tvItem).mask & TVIF_STATE) != 0 {
        (*item).state &= !(*tvItem).stateMask;
        (*item).state |= (*tvItem).state & (*tvItem).stateMask;
    }

    if ((*tvItem).mask & TVIF_STATEEX) != 0 {
        /* FIXME: extended item state not supported (debug trace elided) */
    }

    (*item).callbackMask |= callbackSet;
    (*item).callbackMask &= !callbackClear;

    return TRUE;
}

unsafe extern "C" fn TREEVIEW_InsertItemT(
    mut infoPtr: *mut TREEVIEW_INFO,
    mut ptdi: *const TVINSERTSTRUCTW,
    mut isW: BOOL,
) -> LRESULT {
    let tvItem: *const TVITEMEXW = core::ptr::addr_of!((*ptdi).u.itemex);
    let mut insertAfter: HTREEITEM = core::ptr::null_mut();
    let mut newItem: *mut TREEVIEW_ITEM = core::ptr::null_mut();
    let mut parentItem: *mut TREEVIEW_ITEM = core::ptr::null_mut();
    let mut bTextUpdated: BOOL = FALSE;

    if (*ptdi).hParent == (TVI_ROOT).cast() || (*ptdi).hParent == core::ptr::null_mut()
    {
        parentItem = (*infoPtr).root as *mut TREEVIEW_ITEM;
    } else {
        parentItem = (*ptdi).hParent as *mut TREEVIEW_ITEM;

        if TREEVIEW_ValidItem(infoPtr, parentItem as HTREEITEM) == 0 {
            return 0 as LRESULT;
        }
    }

    insertAfter = ((*ptdi).hInsertAfter).cast();

    /* Validate this now for convenience. */
    match insertAfter {
        TVI_FIRST | TVI_LAST | TVI_SORT => {}
        _ => {
            if TREEVIEW_ValidItem(infoPtr, insertAfter) == 0
                || (*insertAfter).parent != parentItem as HTREEITEM
            {
                insertAfter = TVI_LAST;
            }
        }
    }

    newItem = TREEVIEW_AllocateItem(infoPtr);
    if newItem.is_null() {
        return 0 as LRESULT;
    }

    (*newItem).parent = parentItem as HTREEITEM;
    (*newItem).iIntegral = 1;
    (*newItem).visibleOrder = -(1) as LONG;

    if TREEVIEW_DoSetItemT(infoPtr, newItem, tvItem, isW) == 0 {
        return 0 as LRESULT;
    }

    /* After this point, nothing can fail. (Except for TVI_SORT.) */

    (*infoPtr).uNumItems = (*infoPtr).uNumItems.wrapping_add(1);

    match insertAfter {
        TVI_FIRST => {
            let originalFirst: *mut TREEVIEW_ITEM = (*parentItem).firstChild as *mut TREEVIEW_ITEM;
            TREEVIEW_InsertBefore(
                newItem,
                (*parentItem).firstChild as *mut TREEVIEW_ITEM,
                parentItem,
            );
            if (*infoPtr).firstVisible == originalFirst as HTREEITEM {
                TREEVIEW_SetFirstVisible(infoPtr, newItem, TRUE);
            }
        }

        TVI_LAST => {
            TREEVIEW_InsertAfter(
                newItem,
                (*parentItem).lastChild as *mut TREEVIEW_ITEM,
                parentItem,
            );
        }

        TVI_SORT => {
            let mut aChild: *mut TREEVIEW_ITEM = core::ptr::null_mut();
            let mut previousChild: *mut TREEVIEW_ITEM = core::ptr::null_mut();
            let originalFirst: *mut TREEVIEW_ITEM = (*parentItem).firstChild as *mut TREEVIEW_ITEM;
            let mut bItemInserted: BOOL = FALSE;

            aChild = (*parentItem).firstChild as *mut TREEVIEW_ITEM;

            bTextUpdated = TRUE;
            TREEVIEW_UpdateDispInfo(infoPtr, newItem, TVIF_TEXT);

            /* Iterate the parent children to see where we fit in */
            while !aChild.is_null() {
                let mut comp: INT = 0;

                TREEVIEW_UpdateDispInfo(infoPtr, aChild, TVIF_TEXT);
                comp = lstrcmpW(
                    (*newItem).pszText as LPCWSTR,
                    (*aChild).pszText as LPCWSTR,
                ) as INT;

                if comp < 0 {
                    /* we are smaller than the current one */
                    TREEVIEW_InsertBefore(newItem, aChild, parentItem);
                    if (*infoPtr).firstVisible == originalFirst as HTREEITEM
                        && aChild == originalFirst
                    {
                        TREEVIEW_SetFirstVisible(infoPtr, newItem, TRUE);
                    }
                    bItemInserted = TRUE;
                    break;
                } else if comp > 0 {
                    /* we are bigger than the current one */
                    previousChild = aChild;

                    /* This will help us to exit if there is no more sibling */
                    aChild = (if (*aChild).nextSibling == core::ptr::null_mut() {
                        core::ptr::null_mut()
                    } else {
                        (*aChild).nextSibling
                    }) as *mut TREEVIEW_ITEM;

                    /* Look at the next item */
                    continue;
                } else if comp == 0 {
                    /*
                     * An item with this name is already existing, therefore,
                     * we add after the one we found
                     */
                    TREEVIEW_InsertAfter(newItem, aChild, parentItem);
                    bItemInserted = TRUE;
                    break;
                }
            }

            /*
             * we reach the end of the child list and the item has not
             * yet been inserted, therefore, insert it after the last child.
             */
            if bItemInserted == 0 && aChild.is_null() {
                TREEVIEW_InsertAfter(newItem, previousChild, parentItem);
            }
        }

        /* hInsertAfter names a specific item we want to insert after */
        _ => {
            TREEVIEW_InsertAfter(
                newItem,
                insertAfter as *mut TREEVIEW_ITEM,
                (*insertAfter).parent as *mut TREEVIEW_ITEM,
            );
        }
    }

    (*newItem).iLevel = (*(*newItem).parent).iLevel + 1;

    if (*(*newItem).parent).cChildren == 0 {
        (*(*newItem).parent).cChildren = 1;
    }

    if ((*infoPtr).dwStyle & TVS_CHECKBOXES) != 0 {
        if STATEIMAGEINDEX!((*newItem).state) == 0 {
            (*newItem).state |= INDEXTOSTATEIMAGEMASK!(1);
        }
    }

    if (*infoPtr).firstVisible.is_null() {
        (*infoPtr).firstVisible = newItem as HTREEITEM;
    }

    TREEVIEW_VerifyTree(infoPtr);

    if (*infoPtr).bRedraw == 0 {
        return newItem as LRESULT;
    }

    if parentItem == (*infoPtr).root as *mut TREEVIEW_ITEM
        || (ISVISIBLE!(parentItem) && ((*parentItem).state & TVIS_EXPANDED) != 0)
    {
        let mut item: *mut TREEVIEW_ITEM = core::ptr::null_mut();
        let prev: *mut TREEVIEW_ITEM = TREEVIEW_GetPrevListItem(infoPtr, newItem);

        TREEVIEW_RecalculateVisibleOrder(infoPtr, prev);
        TREEVIEW_ComputeItemInternalMetrics(infoPtr, newItem);

        if bTextUpdated == 0 {
            TREEVIEW_UpdateDispInfo(infoPtr, newItem, TVIF_TEXT);
        }

        TREEVIEW_ComputeTextWidth(infoPtr, newItem, core::ptr::null_mut());
        TREEVIEW_UpdateScrollBars(infoPtr);

        /*
         * if the item was inserted in a visible part of the tree,
         * invalidate it, as well as those after it
         */
        item = newItem;
        while !item.is_null() {
            TREEVIEW_Invalidate(infoPtr, item);
            item = TREEVIEW_GetNextListItem(infoPtr, item);
        }
    } else {
        /* refresh treeview if newItem is the first item inserted under parentItem */
        if ISVISIBLE!(parentItem) && (*newItem).prevSibling == (*newItem).nextSibling {
            /* parent got '+' - update it */
            TREEVIEW_Invalidate(infoPtr, parentItem);
        }
    }

    return newItem as LRESULT;
}

unsafe extern "C" fn TREEVIEW_RemoveAllChildren(infoPtr: *mut TREEVIEW_INFO, parentItem: *const TREEVIEW_ITEM) {
    let mut kill: *mut TREEVIEW_ITEM = (*parentItem).firstChild as *mut TREEVIEW_ITEM;
    while !kill.is_null() {
        let next: *mut TREEVIEW_ITEM = (*kill).nextSibling as *mut TREEVIEW_ITEM;
        TREEVIEW_RemoveItem(infoPtr, kill);
        kill = next;
    }
    assert!(((*parentItem).cChildren <= 0) as i32);
    assert!(((*parentItem).firstChild.is_null()) as i32);
    assert!(((*parentItem).lastChild.is_null()) as i32);
}

unsafe extern "C" fn TREEVIEW_UnlinkItem(item: *const TREEVIEW_ITEM) {
    let mut parentItem: *mut TREEVIEW_ITEM = core::ptr::null_mut();
    assert!(!item.is_null());
    assert!(!(*item).parent.is_null());

    parentItem = (*item).parent as *mut TREEVIEW_ITEM;

    if (*parentItem).firstChild == item as HTREEITEM {
        (*parentItem).firstChild = (*item).nextSibling;
    }

    if (*parentItem).lastChild == item as HTREEITEM {
        (*parentItem).lastChild = (*item).prevSibling;
    }

    if (*parentItem).firstChild.is_null()
        && (*parentItem).lastChild.is_null()
        && (*parentItem).cChildren > 0
    {
        (*parentItem).cChildren = 0;
    }

    if !(*item).prevSibling.is_null() {
        (*(*item).prevSibling).nextSibling = (*item).nextSibling;
    }

    if !(*item).nextSibling.is_null() {
        (*(*item).nextSibling).prevSibling = (*item).prevSibling;
    }
}

unsafe extern "C" fn TREEVIEW_RemoveItem(infoPtr: *mut TREEVIEW_INFO, item: *mut TREEVIEW_ITEM) {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TREEVIEW_RemoveItem\0" as *const u8 as *const ::core::ffi::c_char,
            1488 as ::core::ffi::c_int,
            b"%p, (%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            item,
            TREEVIEW_ItemName(item),
        );
    }
    if !(*item).firstChild.is_null() {
        TREEVIEW_RemoveAllChildren(infoPtr, item);
    }
    TREEVIEW_SendTreeviewNotify(
        infoPtr,
        TVN_DELETEITEMW as UINT,
        TVC_UNKNOWN as UINT,
        (TVIF_HANDLE | TVIF_PARAM) as UINT,
        item as HTREEITEM,
        core::ptr::null_mut(),
    );
    TREEVIEW_UnlinkItem(item);
    (*infoPtr).uNumItems = (*infoPtr).uNumItems.wrapping_sub(1);
    if (*item).pszText != LPSTR_TEXTCALLBACKW {
        heap_free((*item).pszText as *mut ::core::ffi::c_void);
    }
    TREEVIEW_FreeItem(infoPtr, item);
}

unsafe extern "C" fn TREEVIEW_RemoveTree(infoPtr: *mut TREEVIEW_INFO) {
    TREEVIEW_RemoveAllChildren(infoPtr, (*infoPtr).root as *const TREEVIEW_ITEM);

    assert!(((*infoPtr).uNumItems == 0) as i32);	/* root isn't counted in uNumItems */
}

unsafe extern "C" fn TREEVIEW_DeleteItem(infoPtr: *mut TREEVIEW_INFO, item: HTREEITEM) -> LRESULT {
    let mut newSelection: *mut TREEVIEW_ITEM = core::ptr::null_mut::<TREEVIEW_ITEM>();
    let mut newFirstVisible: *mut TREEVIEW_ITEM = core::ptr::null_mut::<TREEVIEW_ITEM>();
    let mut parent: *mut TREEVIEW_ITEM;
    let mut prev: *mut TREEVIEW_ITEM = core::ptr::null_mut::<TREEVIEW_ITEM>();
    let mut visible: BOOL = FALSE;

    if item == TVI_ROOT || item.is_null() {
        parent = (*infoPtr).root as *mut TREEVIEW_ITEM;
        newSelection = core::ptr::null_mut::<TREEVIEW_ITEM>();
        visible = TRUE;
        TREEVIEW_RemoveTree(infoPtr);
    } else {
        if TREEVIEW_ValidItem(infoPtr, item) == 0 {
            return (FALSE) as LRESULT;
        }

        parent = (*item).parent as *mut TREEVIEW_ITEM;

        if (*item).visibleOrder >= 0 {
            prev = TREEVIEW_GetPrevListItem(infoPtr, item as *const TREEVIEW_ITEM);
            visible = TRUE;
        }

        if !(*infoPtr).selectedItem.is_null()
            && (item == (*infoPtr).selectedItem
                || TREEVIEW_IsChildOf(
                    item as *const TREEVIEW_ITEM,
                    (*infoPtr).selectedItem as *const TREEVIEW_ITEM,
                ) != 0)
        {
            if !(*item).nextSibling.is_null() {
                newSelection = (*item).nextSibling as *mut TREEVIEW_ITEM;
            } else if (*item).parent != (*infoPtr).root {
                newSelection = (*item).parent as *mut TREEVIEW_ITEM;
            } else {
                newSelection = (*item).prevSibling as *mut TREEVIEW_ITEM;
            }
        }

        if (*infoPtr).firstVisible == item {
            visible = TRUE;
            if !(*item).nextSibling.is_null() {
                newFirstVisible = (*item).nextSibling as *mut TREEVIEW_ITEM;
            } else if !(*item).prevSibling.is_null() {
                newFirstVisible = (*item).prevSibling as *mut TREEVIEW_ITEM;
            } else if (*item).parent != (*infoPtr).root {
                newFirstVisible = (*item).parent as *mut TREEVIEW_ITEM;
            }
            TREEVIEW_SetFirstVisible(infoPtr, core::ptr::null_mut::<TREEVIEW_ITEM>(), TRUE);
        } else {
            newFirstVisible = (*infoPtr).firstVisible as *mut TREEVIEW_ITEM;
        }

        TREEVIEW_RemoveItem(infoPtr, item as *mut TREEVIEW_ITEM);
    }

    /* Don't change if somebody else already has (infoPtr->selectedItem is cleared by FreeItem). */
    if (*infoPtr).selectedItem.is_null() && !newSelection.is_null() {
        if TREEVIEW_ValidItem(infoPtr, newSelection as HTREEITEM) != 0 {
            TREEVIEW_DoSelectItem(
                infoPtr,
                (TVGN_CARET) as INT,
                newSelection as HTREEITEM,
                (TVC_UNKNOWN) as INT,
            );
        }
    }

    /* Validate insertMark dropItem.
     * hotItem ??? - used for comparison only.
     */
    if TREEVIEW_ValidItem(infoPtr, (*infoPtr).insertMarkItem) == 0 {
        (*infoPtr).insertMarkItem = core::ptr::null_mut();
    }

    if TREEVIEW_ValidItem(infoPtr, (*infoPtr).dropItem) == 0 {
        (*infoPtr).dropItem = core::ptr::null_mut();
    }

    if TREEVIEW_ValidItem(infoPtr, newFirstVisible as HTREEITEM) == 0 {
        newFirstVisible = (*(*infoPtr).root).firstChild as *mut TREEVIEW_ITEM;
    }

    TREEVIEW_VerifyTree(infoPtr);

    if visible != 0 {
        TREEVIEW_SetFirstVisible(infoPtr, newFirstVisible, TRUE);
    }

    if (*infoPtr).bRedraw == 0 {
        return (TRUE) as LRESULT;
    }

    if visible != 0 {
        TREEVIEW_RecalculateVisibleOrder(infoPtr, prev);
        TREEVIEW_UpdateScrollBars(infoPtr);
        TREEVIEW_Invalidate(infoPtr, core::ptr::null::<TREEVIEW_ITEM>());
    } else if (*parent).visibleOrder >= 0 && TREEVIEW_HasChildren(infoPtr, parent) == 0 {
        /* parent lost 0x002B2F2D /* 0x002B2F2D /* 0x002B2F2D /* 0x002B2F2D /* 0x002B2F2D /* 0x002B2F2D /* '+/-' */ */ */ */ */ */ - update it */
        TREEVIEW_Invalidate(infoPtr, parent);
    }

    return (TRUE) as LRESULT;
}

unsafe extern "C" fn TREEVIEW_SetRedraw(mut infoPtr: *mut TREEVIEW_INFO, mut wParam: WPARAM) -> LRESULT {
    (*infoPtr).bRedraw = (wParam != 0 as WPARAM) as BOOL;
    if (*infoPtr).bRedraw != 0 {
        TREEVIEW_UpdateSubTree(infoPtr, (*infoPtr).root as *mut TREEVIEW_ITEM);
        TREEVIEW_RecalculateVisibleOrder(infoPtr, ::core::ptr::null_mut::<TREEVIEW_ITEM>());
        TREEVIEW_UpdateScrollBars(infoPtr);
        TREEVIEW_Invalidate(infoPtr, ::core::ptr::null::<TREEVIEW_ITEM>());
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn TREEVIEW_GetIndent(infoPtr: *const TREEVIEW_INFO) -> LRESULT {
    // TRACE("\n");
    ((*infoPtr).uIndent) as _
}

unsafe extern "C" fn TREEVIEW_SetIndent(infoPtr: *mut TREEVIEW_INFO, mut newIndent: UINT) -> LRESULT {
    if newIndent < MINIMUM_INDENT as UINT {
        newIndent = MINIMUM_INDENT as UINT;
    }

    if (*infoPtr).uIndent != newIndent {
        (*infoPtr).uIndent = newIndent;
        TREEVIEW_UpdateSubTree(infoPtr, (*infoPtr).root as *mut TREEVIEW_ITEM);
        TREEVIEW_UpdateScrollBars(infoPtr);
        TREEVIEW_Invalidate(infoPtr, core::ptr::null::<TREEVIEW_ITEM>());
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TREEVIEW_GetToolTips(infoPtr: *const TREEVIEW_INFO) -> LRESULT {
    return (*infoPtr).hwndToolTip as LRESULT;
}

unsafe extern "C" fn TREEVIEW_SetToolTips(infoPtr: *mut TREEVIEW_INFO, hwndTT: HWND) -> LRESULT {
    let mut prevToolTip: HWND;

    prevToolTip = (*infoPtr).hwndToolTip;
    (*infoPtr).hwndToolTip = hwndTT;

    return prevToolTip as LRESULT;
}

unsafe extern "C" fn TREEVIEW_SetUnicodeFormat(infoPtr: *mut TREEVIEW_INFO, fUnicode: BOOL) -> LRESULT {
    let mut rc: BOOL = (*infoPtr).bNtfUnicode;
    (*infoPtr).bNtfUnicode = fUnicode;
    return rc as LRESULT;
}

unsafe extern "C" fn TREEVIEW_GetUnicodeFormat(infoPtr: *const TREEVIEW_INFO) -> LRESULT {
    ((*infoPtr).bNtfUnicode) as _
}

unsafe extern "C" fn TREEVIEW_GetScrollTime(infoPtr: *const TREEVIEW_INFO) -> LRESULT {
    ((*infoPtr).uScrollTime) as _
}

unsafe extern "C" fn TREEVIEW_SetScrollTime(infoPtr: *mut TREEVIEW_INFO, uScrollTime: UINT) -> LRESULT {
    let uOldScrollTime: UINT = (*infoPtr).uScrollTime;

    (*infoPtr).uScrollTime = min(uScrollTime, 100);

    return uOldScrollTime as LRESULT;
}

unsafe extern "C" fn TREEVIEW_GetImageList(infoPtr: *const TREEVIEW_INFO, wParam: WPARAM) -> LRESULT {
    match wParam {
        x if x == TVSIL_NORMAL as WPARAM => {
            return (*infoPtr).himlNormal as LRESULT;
        }
        x if x == TVSIL_STATE as WPARAM => {
            return (*infoPtr).himlState as LRESULT;
        }
        _ => {
            return 0 as LRESULT;
        }
    }
}

unsafe extern "C" fn TREEVIEW_NaturalHeight(infoPtr: *const TREEVIEW_INFO) -> UINT {
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    let hdc: HDC = GetDC(core::ptr::null_mut());
    let hOldFont: HFONT = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;
    let mut height: UINT;

    /* Height is the maximum of:
     * 16 (a hack because our fonts are tiny), and
     * The text height + border & margin, and
     * The size of the normal image list
     */
    GetTextMetricsW(hdc, core::ptr::addr_of_mut!(tm));
    SelectObject(hdc, hOldFont as HGDIOBJ);
    ReleaseDC(core::ptr::null_mut(), hdc);

    height = (TVHEIGHT_MIN as u32);
    if height
        < (tm.tmHeight as ::core::ffi::c_int
            + tm.tmExternalLeading as ::core::ffi::c_int
            + TVHEIGHT_FONT_ADJUST) as UINT
    {
        height = (tm.tmHeight as ::core::ffi::c_int
            + tm.tmExternalLeading as ::core::ffi::c_int
            + TVHEIGHT_FONT_ADJUST) as UINT;
    }
    if height < (*infoPtr).normalImageHeight as UINT {
        height = (*infoPtr).normalImageHeight as UINT;
    }

    /* Round down, unless we support odd ("non even") heights. */
    if (*infoPtr).dwStyle & TVS_NONEVENHEIGHT == 0 {
        height &= !(1);
    }

    return height;
}

unsafe extern "C" fn TREEVIEW_SetImageList(
    infoPtr: *mut TREEVIEW_INFO,
    type_0: UINT,
    himlNew: HIMAGELIST,
) -> LRESULT {
    let mut himlOld: HIMAGELIST = core::ptr::null_mut();
    let mut oldWidth: ::core::ffi::c_int = (*infoPtr).normalImageWidth;
    let mut oldHeight: ::core::ffi::c_int = (*infoPtr).normalImageHeight;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TREEVIEW_SetImageList\0" as *const u8 as *const ::core::ffi::c_char,
            1770 as ::core::ffi::c_int,
            b"%u,%p\n\0" as *const u8 as *const ::core::ffi::c_char,
            type_0,
            himlNew,
        );
    }
    match type_0 {
        TVSIL_NORMAL => {
            himlOld = (*infoPtr).himlNormal;
            (*infoPtr).himlNormal = himlNew;
            if !himlNew.is_null() {
                ImageList_GetIconSize(
                    himlNew,
                    core::ptr::addr_of_mut!((*infoPtr).normalImageWidth),
                    core::ptr::addr_of_mut!((*infoPtr).normalImageHeight),
                );
            } else {
                (*infoPtr).normalImageWidth = 0;
                (*infoPtr).normalImageHeight = 0;
            }
        }
        TVSIL_STATE => {
            himlOld = (*infoPtr).himlState;
            (*infoPtr).himlState = himlNew;
            if !himlNew.is_null() {
                ImageList_GetIconSize(
                    himlNew,
                    core::ptr::addr_of_mut!((*infoPtr).stateImageWidth),
                    core::ptr::addr_of_mut!((*infoPtr).stateImageHeight),
                );
            } else {
                (*infoPtr).stateImageWidth = 0;
                (*infoPtr).stateImageHeight = 0;
            }
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
                    b"dll/win32/comctl32/treeview.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TREEVIEW_SetImageList\0" as *const u8 as *const ::core::ffi::c_char,
                    1805 as ::core::ffi::c_int,
                    b"unknown imagelist type %u\n\0" as *const u8 as *const ::core::ffi::c_char,
                    type_0,
                );
            }
        }
    }

    if oldWidth != (*infoPtr).normalImageWidth || oldHeight != (*infoPtr).normalImageHeight {
        let mut bRecalcVisible: BOOL = FALSE;

        if oldHeight != (*infoPtr).normalImageHeight && (*infoPtr).bHeightSet == 0 {
            (*infoPtr).uItemHeight = TREEVIEW_NaturalHeight(infoPtr);
            bRecalcVisible = TRUE;
        }

        if (*infoPtr).normalImageWidth > MINIMUM_INDENT
            && (*infoPtr).normalImageWidth as UINT != (*infoPtr).uIndent
        {
            (*infoPtr).uIndent = (*infoPtr).normalImageWidth as UINT;
            bRecalcVisible = TRUE;
        }

        if bRecalcVisible != 0 {
            TREEVIEW_RecalculateVisibleOrder(infoPtr, core::ptr::null_mut::<TREEVIEW_ITEM>());
        }

        TREEVIEW_UpdateSubTree(infoPtr, (*infoPtr).root as *mut TREEVIEW_ITEM);
        TREEVIEW_UpdateScrollBars(infoPtr);
    }

    TREEVIEW_Invalidate(infoPtr, core::ptr::null::<TREEVIEW_ITEM>());

    return himlOld as LRESULT;
}

unsafe extern "C" fn TREEVIEW_SetItemHeight(infoPtr: *mut TREEVIEW_INFO, mut newHeight: INT) -> LRESULT {
    let mut prevHeight: INT = (*infoPtr).uItemHeight as INT;

    if newHeight == -1 as INT {
        (*infoPtr).uItemHeight = TREEVIEW_NaturalHeight(infoPtr) as UINT;
        (*infoPtr).bHeightSet = FALSE as BOOL;
    } else {
        if newHeight == 0 as INT {
            newHeight = 1 as INT;
        }
        (*infoPtr).uItemHeight = newHeight as UINT;
        (*infoPtr).bHeightSet = TRUE as BOOL;
    }

    /* Round down, unless we support odd ("non even") heights. */
    if ((*infoPtr).dwStyle & (TVS_NONEVENHEIGHT as DWORD)) == 0
        && (*infoPtr).uItemHeight != 1 as UINT
    {
        (*infoPtr).uItemHeight &= !(1 as INT) as UINT;
    }

    if (*infoPtr).uItemHeight != prevHeight as UINT {
        TREEVIEW_RecalculateVisibleOrder(infoPtr, core::ptr::null_mut::<TREEVIEW_ITEM>());
        TREEVIEW_UpdateScrollBars(infoPtr);
        TREEVIEW_Invalidate(infoPtr, core::ptr::null::<TREEVIEW_ITEM>());
    }

    return prevHeight as LRESULT;
}

unsafe extern "C" fn TREEVIEW_GetItemHeight(infoPtr: *const TREEVIEW_INFO) -> LRESULT {
    // TRACE("\n");
    ((*infoPtr).uItemHeight) as _
}

unsafe extern "C" fn TREEVIEW_GetFont(infoPtr: *const TREEVIEW_INFO) -> LRESULT {
    return (*infoPtr).hFont as LRESULT;
}

unsafe extern "system" fn TREEVIEW_ResetTextWidth(pItem: LPVOID, unused: LPVOID) -> INT {
    let _ = unused;

    (*(pItem as *mut TREEVIEW_ITEM)).textWidth = 0;

    return 1;
}

unsafe extern "C" fn TREEVIEW_SetFont(mut infoPtr: *mut TREEVIEW_INFO, mut hFont: HFONT, mut bRedraw: BOOL) -> LRESULT {
    let mut uHeight: UINT = (*infoPtr).uItemHeight;

    (*infoPtr).hFont = if !hFont.is_null() {
        hFont
    } else {
        (*infoPtr).hDefaultFont
    };

    DeleteObject((*infoPtr).hBoldFont as HGDIOBJ);
    DeleteObject((*infoPtr).hUnderlineFont as HGDIOBJ);
    DeleteObject((*infoPtr).hBoldUnderlineFont as HGDIOBJ);
    (*infoPtr).hBoldFont = TREEVIEW_CreateBoldFont((*infoPtr).hFont);
    (*infoPtr).hUnderlineFont = TREEVIEW_CreateUnderlineFont((*infoPtr).hFont);
    (*infoPtr).hBoldUnderlineFont = TREEVIEW_CreateBoldUnderlineFont((*infoPtr).hFont);

    if (*infoPtr).bHeightSet == 0 {
        (*infoPtr).uItemHeight = TREEVIEW_NaturalHeight(infoPtr);
    }

    if uHeight != (*infoPtr).uItemHeight {
        TREEVIEW_RecalculateVisibleOrder(infoPtr, ::core::ptr::null_mut::<TREEVIEW_ITEM>());
    }

    DPA_EnumCallback(
        (*infoPtr).items,
        Some(TREEVIEW_ResetTextWidth),
        core::ptr::null_mut(),
    );

    TREEVIEW_UpdateSubTree(infoPtr, (*infoPtr).root as *mut TREEVIEW_ITEM);
    TREEVIEW_UpdateScrollBars(infoPtr);

    if bRedraw != 0 {
        TREEVIEW_Invalidate(infoPtr, ::core::ptr::null::<TREEVIEW_ITEM>());
    }

    return 0;
}

unsafe extern "C" fn TREEVIEW_GetLineColor(infoPtr: *const TREEVIEW_INFO) -> LRESULT {
    return (*infoPtr).clrLine as LRESULT;
}

unsafe extern "C" fn TREEVIEW_SetLineColor(infoPtr: *mut TREEVIEW_INFO, color: COLORREF) -> LRESULT {
    let mut prevColor: COLORREF = (*infoPtr).clrLine;
    (*infoPtr).clrLine = color;
    return prevColor as LRESULT;
}

unsafe extern "C" fn TREEVIEW_GetTextColor(infoPtr: *const TREEVIEW_INFO) -> LRESULT {
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"TREEVIEW_GetTextColor\0" as *const u8 as *const core::ffi::c_char,
            1955 as core::ffi::c_int,
            b"\n\0" as *const u8 as *const core::ffi::c_char,
        );
    }
    return (*infoPtr).clrText as LRESULT;
}

unsafe extern "C" fn TREEVIEW_SetTextColor(
    infoPtr: *mut TREEVIEW_INFO,
    color: COLORREF,
) -> LRESULT {
    let mut prevColor: COLORREF = (*infoPtr).clrText;

    (*infoPtr).clrText = color;

    if (*infoPtr).clrText != prevColor {
        TREEVIEW_Invalidate(infoPtr, core::ptr::null::<TREEVIEW_ITEM>());
    }

    return prevColor as LRESULT;
}

unsafe extern "C" fn TREEVIEW_GetBkColor(infoPtr: *const TREEVIEW_INFO) -> LRESULT {
    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & ((1 as core::ffi::c_int) << (__WINE_DBCL_TRACE as core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"TREEVIEW_GetBkColor\0" as *const u8 as *const core::ffi::c_char,
            1977 as core::ffi::c_int,
            b"\n\0" as *const u8 as *const core::ffi::c_char,
        );
    }
    return (*infoPtr).clrBk as LRESULT;
}

unsafe extern "C" fn TREEVIEW_SetBkColor(infoPtr: *mut TREEVIEW_INFO, newColor: COLORREF) -> LRESULT {
    let mut prevColor: COLORREF = (*infoPtr).clrBk;

    (*infoPtr).clrBk = newColor;

    if newColor != prevColor {
        TREEVIEW_Invalidate(infoPtr, core::ptr::null());
    }

    return prevColor as LRESULT;
}

unsafe extern "C" fn TREEVIEW_GetInsertMarkColor(mut infoPtr: *const TREEVIEW_INFO) -> LRESULT {
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TREEVIEW_GetInsertMarkColor\0" as *const u8 as *const ::core::ffi::c_char,
            1999 as ::core::ffi::c_int,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    return (*infoPtr).clrInsertMark as LRESULT;
}

unsafe extern "C" fn TREEVIEW_SetInsertMarkColor(
    mut infoPtr: *mut TREEVIEW_INFO,
    mut color: COLORREF,
) -> LRESULT {
    let mut prevColor: COLORREF = (*infoPtr).clrInsertMark;
    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & ((1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"TREEVIEW_SetInsertMarkColor\0" as *const u8 as *const core::ffi::c_char,
            2008 as core::ffi::c_int,
            b"0x%08x\n\0" as *const u8 as *const core::ffi::c_char,
            color,
        );
    }
    (*infoPtr).clrInsertMark = color;
    return prevColor as LRESULT;
}

unsafe extern "C" fn TREEVIEW_SetInsertMark(infoPtr: *mut TREEVIEW_INFO, wParam: BOOL, item: HTREEITEM) -> LRESULT {
    if TREEVIEW_ValidItem(infoPtr, item) == 0 {
        return 0;
    }

    (*infoPtr).insertBeforeorAfter = wParam;
    (*infoPtr).insertMarkItem = item;

    TREEVIEW_Invalidate(infoPtr, core::ptr::null::<TREEVIEW_ITEM>());

    return 1;
}

unsafe extern "C" fn TREEVIEW_GetItemRect(
    infoPtr: *const TREEVIEW_INFO,
    fTextRect: BOOL,
    lpRect: LPRECT,
) -> LRESULT {
    let mut item: *mut TREEVIEW_ITEM = core::ptr::null_mut::<TREEVIEW_ITEM>();
    let pItem: *const HTREEITEM = lpRect as *mut HTREEITEM;

    if pItem.is_null() {
        return (FALSE) as LRESULT;
    }

    item = *pItem as *mut TREEVIEW_ITEM;
    if TREEVIEW_ValidItem(infoPtr, item as HTREEITEM) == 0 || ISVISIBLE!(item) == false {
        return (FALSE) as LRESULT;
    }

    if fTextRect != 0 {
        (*lpRect).top = (*item).rect.top;
        (*lpRect).bottom = (*item).rect.bottom;

        (*lpRect).left = (*item).textOffset;
        if (*item).textWidth == 0 {
            TREEVIEW_ComputeTextWidth(infoPtr, item, core::ptr::null_mut());
        }

        (*lpRect).right = ((*item).textOffset as core::ffi::c_int
            + (*item).textWidth as core::ffi::c_int
            + 4) as LONG;
    } else {
        *lpRect = (*item).rect;
    }

    return (TRUE) as LRESULT;
}

unsafe extern "C" fn TREEVIEW_GetVisibleCount(infoPtr: *const TREEVIEW_INFO) -> LRESULT {
    return ((*infoPtr).clientHeight as UINT).wrapping_div((*infoPtr).uItemHeight) as LRESULT;
}

unsafe extern "C" fn TREEVIEW_GetItemT(
    mut infoPtr: *const TREEVIEW_INFO,
    mut tvItem: LPTVITEMEXW,
    mut isW: BOOL,
) -> LRESULT {
    let mut item: *mut TREEVIEW_ITEM = (*tvItem).hItem as *mut TREEVIEW_ITEM;
    if TREEVIEW_ValidItem(infoPtr, item as HTREEITEM) == 0 {
        let mut valid_item: BOOL = FALSE;
        if item.is_null() {
            return FALSE as LRESULT;
        }
        infoPtr = (*item).infoPtr;
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/treeview.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TREEVIEW_GetItemT\0" as *const u8 as *const ::core::ffi::c_char,
                2100 as ::core::ffi::c_int,
                b"got item from different tree %p, called from %p\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                (*item).infoPtr,
                infoPtr,
            );
        }
        valid_item = TREEVIEW_ValidItem(infoPtr, item as HTREEITEM);
        if valid_item == 0 {
            return FALSE as LRESULT;
        }
    }
    TREEVIEW_UpdateDispInfo(infoPtr, item, (*tvItem).mask);
    if (*tvItem).mask & TVIF_CHILDREN as UINT != 0 {
        if (*item).cChildren == I_CHILDRENCALLBACK {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/treeview.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TREEVIEW_GetItemT\0" as *const u8 as *const ::core::ffi::c_char,
                    2115 as ::core::ffi::c_int,
                    b"I_CHILDRENCALLBACK not supported\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
        }
        (*tvItem).cChildren = (*item).cChildren;
    }
    if (*tvItem).mask & TVIF_HANDLE as UINT != 0 {
        (*tvItem).hItem = (item as HTREEITEM).cast();
    }
    if (*tvItem).mask & TVIF_IMAGE as UINT != 0 {
        (*tvItem).iImage = (*item).iImage;
    }
    if (*tvItem).mask & TVIF_INTEGRAL as UINT != 0 {
        (*tvItem).iIntegral = (*item).iIntegral;
    }
    (*tvItem).lParam = (*item).lParam;
    if (*tvItem).mask & TVIF_SELECTEDIMAGE as UINT != 0 {
        (*tvItem).iSelectedImage = (*item).iSelectedImage;
    }
    if (*tvItem).mask & TVIF_EXPANDEDIMAGE as UINT != 0 {
        (*tvItem).iExpandedImage = (*item).iExpandedImage;
    }
    (*tvItem).state = (*item).state;
    if (*tvItem).mask & TVIF_TEXT as UINT != 0 {
        if (*item).pszText.is_null() {
            if (*tvItem).cchTextMax > 0 as ::core::ffi::c_int {
                *(*tvItem).pszText.offset(0 as ::core::ffi::c_int as isize) = '\0' as i32 as WCHAR;
            }
        } else if isW != 0 {
            if (*item).pszText == LPSTR_TEXTCALLBACKW {
                (*tvItem).pszText = LPSTR_TEXTCALLBACKW;
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_1: __wine_debug_class = __WINE_DBCL_FIXME;
                    ros_dbg_log(
                        __dbcl_1,
                        __dbch_1,
                        b"dll/win32/comctl32/treeview.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"TREEVIEW_GetItemT\0" as *const u8
                            as *const ::core::ffi::c_char,
                        2152 as ::core::ffi::c_int,
                        b" GetItem called with LPSTR_TEXTCALLBACK\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                }
            } else {
                lstrcpynW(
                    (*tvItem).pszText,
                    (*item).pszText as LPCWSTR,
                    (*tvItem).cchTextMax,
                );
            }
        } else if (*item).pszText == LPSTR_TEXTCALLBACKW {
            (*tvItem).pszText = LPSTR_TEXTCALLBACKA as LPWSTR;
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                != 0
            {
                let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_2: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl_2,
                    __dbch_2,
                    b"dll/win32/comctl32/treeview.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TREEVIEW_GetItemT\0" as *const u8 as *const ::core::ffi::c_char,
                    2164 as ::core::ffi::c_int,
                    b" GetItem called with LPSTR_TEXTCALLBACK\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
        } else {
            WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                (*item).pszText as LPCWSTR,
                -(1 as ::core::ffi::c_int),
                (*tvItem).pszText as LPSTR,
                (*tvItem).cchTextMax,
                ::core::ptr::null::<CHAR>(),
                ::core::ptr::null_mut::<BOOL>(),
            );
        }
    }
    if (*tvItem).mask & TVIF_STATEEX as UINT != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_3: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl_3,
                __dbch_3,
                b"dll/win32/comctl32/treeview.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TREEVIEW_GetItemT\0" as *const u8 as *const ::core::ffi::c_char,
                2176 as ::core::ffi::c_int,
                b"Extended item state not supported, returning 0.\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        (*tvItem).uStateEx = 0 as UINT;
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_4: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_4,
            __dbch_4,
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TREEVIEW_GetItemT\0" as *const u8 as *const ::core::ffi::c_char,
            2181 as ::core::ffi::c_int,
            b"item <%p>, txt %p, img %d, mask 0x%x\n\0" as *const u8
                as *const ::core::ffi::c_char,
            item,
            (*tvItem).pszText,
            (*tvItem).iImage,
            (*tvItem).mask,
        );
    }
    return TRUE as LRESULT;
}

unsafe extern "C" fn TREEVIEW_SetItemT(infoPtr: *mut TREEVIEW_INFO, tvItem: *const TVITEMEXW, isW: BOOL) -> LRESULT {
    let mut item: *mut TREEVIEW_ITEM;
    let mut originalItem: TREEVIEW_ITEM = core::mem::zeroed();

    item = (*tvItem).hItem as *mut TREEVIEW_ITEM;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TREEVIEW_SetItemT\0" as *const u8 as *const ::core::ffi::c_char,
            2197 as ::core::ffi::c_int,
            b"item %d, mask 0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
            TREEVIEW_GetItemIndex(infoPtr, item as HTREEITEM),
            (*tvItem).mask,
        );
    }

    if ((TREEVIEW_ValidItem(infoPtr, item as HTREEITEM)) == 0) {
        return FALSE as LRESULT;
    }

    /* Store the original item values. Text buffer will be freed in TREEVIEW_DoSetItemT() below. */
    originalItem = core::ptr::read(item);

    if TREEVIEW_DoSetItemT(infoPtr, item, tvItem, isW) == 0 {
        return FALSE as LRESULT;
    }

    /* If the text or TVIS_BOLD was changed, and it is visible, recalculate. */
    if (((*tvItem).mask & (TVIF_TEXT as UINT)) != 0
        || (((*tvItem).mask & (TVIF_STATE as UINT)) != 0
            && ((*tvItem).stateMask & (TVIS_BOLD as UINT)) != 0))
        && ISVISIBLE!(item) != false
    {
        TREEVIEW_UpdateDispInfo(infoPtr, item, TVIF_TEXT as UINT);
        TREEVIEW_ComputeTextWidth(infoPtr, item, core::ptr::null_mut());
    }

    if (*tvItem).mask != (0 as UINT) && ISVISIBLE!(item) != false {
        /* The refresh updates everything, but we can't wait until then. */
        TREEVIEW_ComputeItemInternalMetrics(infoPtr, item);

        /* if any of the item's values changed and it's not a callback, redraw the item */
        if item_changed(core::ptr::addr_of_mut!(originalItem), item, tvItem) != 0 {
            if ((*tvItem).mask & (TVIF_INTEGRAL as UINT)) != 0 {
                TREEVIEW_RecalculateVisibleOrder(infoPtr, item);
                TREEVIEW_UpdateScrollBars(infoPtr);

                TREEVIEW_Invalidate(infoPtr, core::ptr::null_mut());
            } else {
                TREEVIEW_UpdateScrollBars(infoPtr);
                TREEVIEW_Invalidate(infoPtr, item);
            }
        }
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn TREEVIEW_GetItemState(infoPtr: *const TREEVIEW_INFO, item: HTREEITEM, mask: UINT) -> LRESULT {
    if item.is_null() || TREEVIEW_ValidItem(infoPtr, item) == 0 {
        return 0 as LRESULT;
    }

    return ((*item).state & mask) as LRESULT;
}

unsafe extern "C" fn TREEVIEW_GetNextItem(
    mut infoPtr: *const TREEVIEW_INFO,
    mut which: UINT,
    mut item: HTREEITEM,
) -> LRESULT {
    let mut retval: *mut TREEVIEW_ITEM = ::core::ptr::null_mut::<TREEVIEW_ITEM>();
    retval = core::ptr::null_mut();

    /* handle all the global data here */
    let mut fall_to_root: bool = false;
    match which {
        TVGN_CHILD => {
            /* Special case: child of 0 is root */
            if !item.is_null() {
                /* break */
            } else {
                fall_to_root = true;
            }
        }
        TVGN_ROOT => {
            fall_to_root = true;
        }
        TVGN_CARET => {
            retval = (*infoPtr).selectedItem as *mut TREEVIEW_ITEM;
        }
        TVGN_FIRSTVISIBLE => {
            retval = (*infoPtr).firstVisible as *mut TREEVIEW_ITEM;
        }
        TVGN_DROPHILITE => {
            retval = (*infoPtr).dropItem as *mut TREEVIEW_ITEM;
        }
        TVGN_LASTVISIBLE => {
            retval = TREEVIEW_GetLastListItem(infoPtr, (*infoPtr).root as *mut TREEVIEW_ITEM);
        }
        _ => {}
    }
    if fall_to_root {
        retval = (*(*infoPtr).root).firstChild as *mut TREEVIEW_ITEM;
    }

    if !retval.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/treeview.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TREEVIEW_GetNextItem\0" as *const u8 as *const ::core::ffi::c_char,
                2291 as ::core::ffi::c_int,
                b"flags:0x%x, returns %p\n\0" as *const u8 as *const ::core::ffi::c_char,
                which,
                retval,
            );
        }
        return retval as LRESULT;
    }

    if item == TVI_ROOT {
        item = (*infoPtr).root;
    }

    if TREEVIEW_ValidItem(infoPtr, item) == 0 {
        return FALSE as LRESULT;
    }

    match which {
        TVGN_NEXT => {
            retval = (*item).nextSibling as *mut TREEVIEW_ITEM;
        }
        TVGN_PREVIOUS => {
            retval = (*item).prevSibling as *mut TREEVIEW_ITEM;
        }
        TVGN_PARENT => {
            retval = (if (*item).parent != (*infoPtr).root {
                (*item).parent
            } else {
                ::core::ptr::null_mut::<_TREEITEM>()
            }) as *mut TREEVIEW_ITEM;
        }
        TVGN_CHILD => {
            retval = (*item).firstChild as *mut TREEVIEW_ITEM;
        }
        TVGN_NEXTVISIBLE => {
            retval = TREEVIEW_GetNextListItem(infoPtr, item as *const TREEVIEW_ITEM);
        }
        TVGN_PREVIOUSVISIBLE => {
            retval = TREEVIEW_GetPrevListItem(infoPtr, item as *const TREEVIEW_ITEM);
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/treeview.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TREEVIEW_GetNextItem\0" as *const u8 as *const ::core::ffi::c_char,
                    2321 as ::core::ffi::c_int,
                    b"Unknown msg 0x%x, item %p\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    which,
                    item,
                );
            }
        }
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
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TREEVIEW_GetNextItem\0" as *const u8 as *const ::core::ffi::c_char,
            2325 as ::core::ffi::c_int,
            b"flags: 0x%x, item %p;returns %p\n\0" as *const u8
                as *const ::core::ffi::c_char,
            which,
            item,
            retval,
        );
    }
    return retval as LRESULT;
}

unsafe extern "C" fn TREEVIEW_GetCount(infoPtr: *const TREEVIEW_INFO) -> LRESULT {
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const i8,
            b"TREEVIEW_GetCount\0" as *const u8 as *const i8,
            2333i32,
            b" %d\n\0" as *const u8 as *const i8,
            (*infoPtr).uNumItems,
        );
    }
    return (*infoPtr).uNumItems as LRESULT;
}

unsafe extern "C" fn TREEVIEW_ToggleItemState(mut infoPtr: *mut TREEVIEW_INFO, mut item: *mut TREEVIEW_ITEM) {
    if ((*infoPtr).dwStyle & TVS_CHECKBOXES) != 0 {
        static state_table: [u32; 3] = [0, 2, 1];

        let mut state: u32;

        state = STATEIMAGEINDEX!((*item).state) as u32;

        (*item).state &= !(TVIS_STATEIMAGEMASK);

        if state < 3 {
            state = state_table[state as usize];
        }

        (*item).state |= INDEXTOSTATEIMAGEMASK!(state) as u32;

        TREEVIEW_SelectItem(infoPtr, (TVGN_CARET) as INT, item as HTREEITEM);
        TREEVIEW_Invalidate(infoPtr, item);
    }
}

unsafe extern "C" fn TREEVIEW_DrawItemLines(infoPtr: *const TREEVIEW_INFO, hdc: HDC, item: *const TREEVIEW_ITEM) {
    let mut centerx: LONG = 0;
    let mut centery: LONG = 0;
    let lar: BOOL = ((((*infoPtr).dwStyle & (TVS_LINESATROOT | TVS_HASLINES | TVS_HASBUTTONS)) > TVS_LINESATROOT) as ::core::ffi::c_int) as BOOL;
    let mut hbr: HBRUSH = core::ptr::null_mut();
    let mut hbrOld: HBRUSH = core::ptr::null_mut();
    let clrBk: COLORREF = GETBKCOLOR!((*infoPtr).clrBk);

    if lar == 0 && (*item).iLevel == 0 {
        return;
    }

    hbr = CreateSolidBrush(clrBk);
    hbrOld = SelectObject(hdc, hbr as HGDIOBJ) as HBRUSH;

    centerx = (((*item).linesOffset + (*item).stateOffset) / 2) as LONG;
    centery = (((*item).rect.top + (*item).rect.bottom) / 2) as LONG;

    if ((*infoPtr).dwStyle & TVS_HASLINES) != 0 {
        let mut hOldPen: HPEN = core::ptr::null_mut();
        let mut hNewPen: HPEN = core::ptr::null_mut();
        let mut parent: HTREEITEM = core::ptr::null_mut();
        let mut lb: LOGBRUSH = core::mem::zeroed();

        /* Get a dotted grey pen */
        lb.lbStyle = BS_SOLID as UINT;
        lb.lbColor = GETLINECOLOR!((*infoPtr).clrLine);
        hNewPen = ExtCreatePen(
            PS_COSMETIC | PS_ALTERNATE,
            1,
            core::ptr::addr_of_mut!(lb),
            0,
            core::ptr::null(),
        );
        hOldPen = SelectObject(hdc, hNewPen as HGDIOBJ) as HPEN;

        /* Make sure the center is on a dot (using +2 instead
         * of +1 gives us pixel-by-pixel compat with native) */
        centery = (centery + 2) & !1;

        MoveToEx(hdc, (*item).stateOffset, centery, core::ptr::null_mut());
        LineTo(hdc, centerx - 1, centery);

        if !(*item).prevSibling.is_null() || (*item).parent != (*infoPtr).root {
            MoveToEx(hdc, centerx, (*item).rect.top, core::ptr::null_mut());
            LineTo(hdc, centerx, centery);
        }

        if !(*item).nextSibling.is_null() {
            MoveToEx(hdc, centerx, centery, core::ptr::null_mut());
            LineTo(hdc, centerx, (*item).rect.bottom + 1);
        }

        /* Draw the line from our parent to its next sibling. */
        parent = (*item).parent;
        while parent != (*infoPtr).root {
            let pcenterx: ::core::ffi::c_int = (((*parent).linesOffset + (*parent).stateOffset) / 2) as ::core::ffi::c_int;

            if !(*parent).nextSibling.is_null()
                /* skip top-levels unless TVS_LINESATROOT */
                && (*parent).stateOffset > (*parent).linesOffset
            {
                MoveToEx(hdc, pcenterx, (*item).rect.top, core::ptr::null_mut());
                LineTo(hdc, pcenterx, (*item).rect.bottom + 1);
            }

            parent = (*parent).parent;
        }

        SelectObject(hdc, hOldPen as HGDIOBJ);
        DeleteObject(hNewPen as HGDIOBJ);
    }

    /*
     * Display the (+/-) signs
     */

    if ((*infoPtr).dwStyle & TVS_HASBUTTONS) != 0 {
        if (*item).cChildren != 0 {
            let theme: HTHEME = GetWindowTheme((*infoPtr).hwnd);
            if !theme.is_null() {
                let mut glyphRect: RECT = (*item).rect;
                glyphRect.left = (*item).linesOffset;
                glyphRect.right = (*item).stateOffset;
                DrawThemeBackground(
                    theme,
                    hdc,
                    (TVP_GLYPH) as i32,
                    if ((*item).state & TVIS_EXPANDED) != 0 { (GLPS_OPENED) as i32 } else { (GLPS_CLOSED) as i32 },
                    core::ptr::addr_of_mut!(glyphRect),
                    core::ptr::null(),
                );
            } else {
                let height: LONG = (*item).rect.bottom - (*item).rect.top;
                let width: LONG = (*item).stateOffset - (*item).linesOffset;
                let rectsize: LONG = min(height, width) / 4;
                /* plussize = ceil(rectsize * 3/4) */
                let plussize: LONG = (rectsize + 1) * 3 / 4;

                let mut new_pen: HPEN = CreatePen(
                    (PS_SOLID as i32),
                    0,
                    GETLINECOLOR!((*infoPtr).clrLine),
                );
                let mut old_pen: HPEN = SelectObject(hdc, new_pen as HGDIOBJ) as HPEN;

                Rectangle(
                    hdc,
                    centerx - rectsize - 1,
                    centery - rectsize - 1,
                    centerx + rectsize + 2,
                    centery + rectsize + 2,
                );

                SelectObject(hdc, old_pen as HGDIOBJ);
                DeleteObject(new_pen as HGDIOBJ);

                /* draw +/- signs with current text color */
                new_pen = CreatePen(
                    (PS_SOLID as i32),
                    0,
                    GETTXTCOLOR!((*infoPtr).clrText),
                );
                old_pen = SelectObject(hdc, new_pen as HGDIOBJ) as HPEN;

                if height < 18 || width < 18 {
                    MoveToEx(
                        hdc,
                        centerx - plussize + 1,
                        centery,
                        core::ptr::null_mut(),
                    );
                    LineTo(hdc, centerx + plussize, centery);

                    if ((*item).state & TVIS_EXPANDED) == 0
                        || ((*item).state & TVIS_EXPANDPARTIAL) != 0
                    {
                        MoveToEx(
                            hdc,
                            centerx,
                            centery - plussize + 1,
                            core::ptr::null_mut(),
                        );
                        LineTo(hdc, centerx, centery + plussize);
                    }
                } else {
                    Rectangle(
                        hdc,
                        centerx - plussize + 1,
                        centery - 1,
                        centerx + plussize,
                        centery + 2,
                    );

                    if ((*item).state & TVIS_EXPANDED) == 0
                        || ((*item).state & TVIS_EXPANDPARTIAL) != 0
                    {
                        Rectangle(
                            hdc,
                            centerx - 1,
                            centery - plussize + 1,
                            centerx + 2,
                            centery + plussize,
                        );
                        SetPixel(hdc, centerx - 1, centery, clrBk);
                        SetPixel(hdc, centerx + 1, centery, clrBk);
                    }
                }

                SelectObject(hdc, old_pen as HGDIOBJ);
                DeleteObject(new_pen as HGDIOBJ);
            }
        }
    }
    SelectObject(hdc, hbrOld as HGDIOBJ);
    DeleteObject(hbr as HGDIOBJ);
}

unsafe extern "C" fn TREEVIEW_DrawItem(infoPtr: *const TREEVIEW_INFO, hdc: HDC, item: *mut TREEVIEW_ITEM) {
    let mut cditem: INT = 0;
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut oldTextColor: COLORREF = 0;
    let mut oldTextBkColor: COLORREF = 0;
    let mut centery: ::core::ffi::c_int = 0;
    let inFocus: BOOL = (GetFocus() == (*infoPtr).hwnd) as BOOL;
    let mut nmcdhdr: NMTVCUSTOMDRAW = core::mem::zeroed();

    TREEVIEW_UpdateDispInfo(infoPtr, item, CALLBACK_MASK_ALL as UINT);

    /* - If item is drop target or it is selected and window is in focus -
     * use blue background (COLOR_HIGHLIGHT).
     * - If item is selected, window is not in focus, but it has style
     * TVS_SHOWSELALWAYS - use grey background (COLOR_BTNFACE)
     * - Otherwise - use background color
     */
    if ((*item).state & TVIS_DROPHILITED as UINT != 0)
        || (item == ((*infoPtr).focusedItem).cast() && (*item).state & TVIS_SELECTED as UINT == 0)
        || ((*item).state & TVIS_SELECTED as UINT != 0
            && ((*infoPtr).focusedItem.is_null() || item == ((*infoPtr).focusedItem).cast())
            && (inFocus != 0 || (*infoPtr).dwStyle & TVS_SHOWSELALWAYS as DWORD != 0))
    {
        if (*item).state & TVIS_DROPHILITED as UINT != 0 || inFocus != 0 {
            nmcdhdr.clrTextBk = comctl32_color.clrHighlight;
            nmcdhdr.clrText = comctl32_color.clrHighlightText;
        } else {
            nmcdhdr.clrTextBk = comctl32_color.clrBtnFace;
            nmcdhdr.clrText = if (*infoPtr).clrText as ::core::ffi::c_ulong == CLR_NONE {
                comctl32_color.clrWindowText
            } else {
                (*infoPtr).clrText
            };
        }
    } else {
        nmcdhdr.clrTextBk = if (*infoPtr).clrBk as ::core::ffi::c_ulong == CLR_NONE {
            comctl32_color.clrWindow
        } else {
            (*infoPtr).clrBk
        };
        if (*infoPtr).dwStyle & TVS_TRACKSELECT as DWORD != 0 && item == ((*infoPtr).hotItem).cast() {
            nmcdhdr.clrText = comctl32_color.clrHighlight;
        } else {
            nmcdhdr.clrText = if (*infoPtr).clrText as ::core::ffi::c_ulong == CLR_NONE {
                comctl32_color.clrWindowText
            } else {
                (*infoPtr).clrText
            };
        }
    }

    hOldFont = SelectObject(hdc, TREEVIEW_FontForItem(infoPtr, item) as HGDIOBJ) as HFONT;
    oldTextColor = SetTextColor(hdc, nmcdhdr.clrText);
    oldTextBkColor = SetBkColor(hdc, nmcdhdr.clrTextBk);

    /* The custom draw handler can query the text rectangle,
     * so get ready. */
    /* should already be known, set to 0 when changed */
    if (*item).textWidth == 0 {
        TREEVIEW_ComputeTextWidth(infoPtr, item, hdc);
    }

    cditem = 0 as INT;

    if (*infoPtr).cdmode & CDRF_NOTIFYITEMDRAW as INT != 0 {
        cditem = TREEVIEW_SendCustomDrawItemNotify(
            infoPtr,
            hdc,
            item,
            CDDS_ITEMPREPAINT as UINT,
            &raw mut nmcdhdr,
        ) as INT;

        if cditem & CDRF_SKIPDEFAULT as INT != 0 {
            SelectObject(hdc, hOldFont as HGDIOBJ);
            return;
        }
    }

    if cditem & CDRF_NEWFONT as INT != 0 {
        TREEVIEW_ComputeTextWidth(infoPtr, item, hdc);
    }

    if TREEVIEW_IsFullRowSelect(infoPtr) != 0 {
        let brush: HBRUSH = CreateSolidBrush(nmcdhdr.clrTextBk);
        FillRect(hdc, &raw mut (*item).rect as LPCRECT, brush);
        DeleteObject(brush as HGDIOBJ);
    }

    TREEVIEW_DrawItemLines(infoPtr, hdc, item);

    /* reset colors. Custom draw handler can change them */
    SetTextColor(hdc, nmcdhdr.clrText);
    SetBkColor(hdc, nmcdhdr.clrTextBk);

    centery = ((*item).rect.top as ::core::ffi::c_int + (*item).rect.bottom as ::core::ffi::c_int)
        / 2;

    /*
     * Display the images associated with this item
     */
    {
        let mut imageIndex: INT = 0;

        /* State images are displayed to the left of the Normal image
         * image number is in state; zero should be `display no image'.
         */
        imageIndex = (((*item).state >> 12) & 0x0f) as INT;

        if !(*infoPtr).himlState.is_null() && imageIndex != 0 {
            ImageList_Draw(
                (*infoPtr).himlState,
                imageIndex as ::core::ffi::c_int,
                hdc,
                (*item).stateOffset as ::core::ffi::c_int,
                centery - (*infoPtr).stateImageHeight / 2,
                ILD_NORMAL as UINT,
            );
        }

        /* Now, draw the normal image; can be either selected,
         * non-selected or expanded image.
         */

        if (*item).state & TVIS_SELECTED as UINT != 0
            && (*item).iSelectedImage >= 0
        {
            /* The item is currently selected */
            imageIndex = (*item).iSelectedImage as INT;
        } else if (*item).state & TVIS_EXPANDED as UINT != 0
            && (*item).iExpandedImage != (I_IMAGENONE as WORD) as ::core::ffi::c_int
        {
            /* The item is currently not selected but expanded */
            imageIndex = (*item).iExpandedImage as INT;
        } else {
            /* The item is not selected and not expanded */
            imageIndex = (*item).iImage as INT;
        }

        if !(*infoPtr).himlNormal.is_null() {
            let mut style: UINT = if (*item).state & TVIS_CUT as UINT != 0 {
                ILD_SELECTED as UINT
            } else {
                ILD_NORMAL as UINT
            };

            style |= (*item).state & TVIS_OVERLAYMASK as UINT;

            ImageList_DrawEx(
                (*infoPtr).himlNormal,
                imageIndex as ::core::ffi::c_int,
                hdc,
                (*item).imageOffset as ::core::ffi::c_int,
                centery - (*infoPtr).normalImageHeight / 2,
                0,
                0,
                if TREEVIEW_IsFullRowSelect(infoPtr) != 0 {
                    nmcdhdr.clrTextBk
                } else {
                    (*infoPtr).clrBk
                },
                if (*item).state & TVIS_CUT as UINT != 0 {
                    (if (*infoPtr).clrBk as ::core::ffi::c_ulong == CLR_NONE {
                        comctl32_color.clrWindow
                    } else {
                        (*infoPtr).clrBk
                    }) as COLORREF
                } else {
                    CLR_DEFAULT as COLORREF
                },
                style,
            );
        }
    }

    /*
     * Display the text associated with this item
     */

    /* Don't paint item's text if it's being edited */
    if (*infoPtr).hwndEdit.is_null() || (*infoPtr).selectedItem != (item).cast() {
        if !(*item).pszText.is_null() {
            let mut rcText: RECT = core::mem::zeroed();
            let mut align: UINT = 0;
            let mut sz: SIZE = core::mem::zeroed();

            rcText.top = (*item).rect.top;
            rcText.bottom = (*item).rect.bottom;
            rcText.left = (*item).textOffset;
            rcText.right = (rcText.left as ::core::ffi::c_int
                + (*item).textWidth as ::core::ffi::c_int
                + 4) as LONG;

            /* Draw it */
            GetTextExtentPoint32W(
                hdc,
                (*item).pszText as LPCWSTR,
                lstrlenW((*item).pszText as LPCWSTR),
                &raw mut sz,
            );

            align = SetTextAlign(hdc, (TA_LEFT | TA_TOP) as UINT);
            ExtTextOutW(
                hdc,
                rcText.left as ::core::ffi::c_int + 2,
                (rcText.top as ::core::ffi::c_int + rcText.bottom as ::core::ffi::c_int
                    - sz.cy as ::core::ffi::c_int)
                    / 2,
                (ETO_CLIPPED | ETO_OPAQUE) as UINT,
                &raw mut rcText,
                (*item).pszText as LPCWSTR,
                lstrlenW((*item).pszText as LPCWSTR) as UINT,
                core::ptr::null::<INT>(),
            );
            SetTextAlign(hdc, align);

            /* Draw focus box around the selected item */
            if item == ((*infoPtr).selectedItem).cast() && inFocus != 0 {
                DrawFocusRect(hdc, &raw mut rcText as LPCRECT);
            }
        }
    }

    /* Draw insertion mark if necessary */

    if !(*infoPtr).insertMarkItem.is_null() {
        /* TRACE("item:%d,mark:%p\n", TREEVIEW_GetItemIndex(infoPtr, item), infoPtr->insertMarkItem); */
        let _trace_index: i32 = TREEVIEW_GetItemIndex(infoPtr, item as HTREEITEM);
        let _trace_mark: HTREEITEM = (*infoPtr).insertMarkItem;
    }

    if item == ((*infoPtr).insertMarkItem).cast() {
        let mut hNewPen: HPEN = core::ptr::null_mut();
        let mut hOldPen: HPEN = core::ptr::null_mut();
        let mut offset: ::core::ffi::c_int = 0;
        let mut left: ::core::ffi::c_int = 0;
        let mut right: ::core::ffi::c_int = 0;

        hNewPen = CreatePen(
            (PS_SOLID as UINT) as i32,
            2 as INT,
            if (*infoPtr).clrInsertMark as ::core::ffi::c_ulong == CLR_DEFAULT {
                comctl32_color.clrBtnText
            } else {
                (*infoPtr).clrInsertMark
            },
        );
        hOldPen = SelectObject(hdc, hNewPen as HGDIOBJ) as HPEN;

        if (*infoPtr).insertBeforeorAfter != 0 {
            offset = (*item).rect.bottom as ::core::ffi::c_int - 1;
        } else {
            offset = (*item).rect.top as ::core::ffi::c_int + 1;
        }

        left = (*item).textOffset as ::core::ffi::c_int - 2;
        right = (*item).textOffset as ::core::ffi::c_int
            + (*item).textWidth as ::core::ffi::c_int
            + 2;

        MoveToEx(
            hdc,
            left,
            offset - 3,
            core::ptr::null_mut::<tagPOINT>(),
        );
        LineTo(hdc, left, offset + 4);

        MoveToEx(hdc, left, offset, core::ptr::null_mut::<tagPOINT>());
        LineTo(hdc, right + 1, offset);

        MoveToEx(
            hdc,
            right,
            offset + 3,
            core::ptr::null_mut::<tagPOINT>(),
        );
        LineTo(hdc, right, offset - 4);

        SelectObject(hdc, hOldPen as HGDIOBJ);
        DeleteObject(hNewPen as HGDIOBJ);
    }

    /* Restore the hdc state */
    SetTextColor(hdc, oldTextColor);
    SetBkColor(hdc, oldTextBkColor);
    SelectObject(hdc, hOldFont as HGDIOBJ);

    if cditem & CDRF_NOTIFYPOSTPAINT as INT != 0 {
        cditem = TREEVIEW_SendCustomDrawItemNotify(
            infoPtr,
            hdc,
            item,
            CDDS_ITEMPOSTPAINT as UINT,
            &raw mut nmcdhdr,
        ) as INT;
    }
}

unsafe extern "C" fn TREEVIEW_UpdateScrollBars(infoPtr: *mut TREEVIEW_INFO) {
    let mut item: *mut TREEVIEW_ITEM = core::ptr::null_mut();
    let hwnd: HWND = (*infoPtr).hwnd;
    let mut vert: BOOL = FALSE;
    let mut horz: BOOL = FALSE;
    let mut si: SCROLLINFO = core::mem::zeroed();
    let mut scrollX: LONG = (*infoPtr).scrollX;

    (*infoPtr).treeWidth = 0;
    (*infoPtr).treeHeight = 0;

    item = ((*(*infoPtr).root).firstChild).cast();

    while !item.is_null() {
        if ISVISIBLE!(item) {
            if 2 + (*item).textOffset + (*item).textWidth > (*infoPtr).treeWidth {
                (*infoPtr).treeWidth = (*item).textOffset + (*item).textWidth + 2;
            }
            (*infoPtr).treeHeight = (*item).rect.bottom;
        }

        item = TREEVIEW_GetNextListItem(infoPtr, item);
    }

    if !(*(*infoPtr).root).firstChild.is_null() {
        (*infoPtr).treeHeight -= (*(*(*infoPtr).root).firstChild).rect.top;
    }

    (*infoPtr).treeWidth += (*infoPtr).scrollX;

    if ((*infoPtr).dwStyle & TVS_NOSCROLL) != 0 {
        return;
    }

    if (*infoPtr).treeHeight > (*infoPtr).clientHeight {
        vert = TRUE;

        if (*infoPtr).treeWidth
            > (*infoPtr).clientWidth - GetSystemMetrics(SM_CXVSCROLL as i32)
        {
            horz = TRUE;
        }
    } else if (*infoPtr).treeWidth > (*infoPtr).clientWidth
        || (*infoPtr).scrollX > 0
    {
        horz = TRUE;
    }

    if vert == 0
        && horz != 0
        && (*infoPtr).treeHeight
            > (*infoPtr).clientHeight - GetSystemMetrics(SM_CYVSCROLL as i32)
    {
        vert = TRUE;
    }

    if horz != 0 && ((*infoPtr).dwStyle & TVS_NOHSCROLL) != 0 {
        horz = FALSE;
    }

    si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    si.fMask = (SIF_POS | SIF_RANGE | SIF_PAGE) as UINT;
    si.nMin = 0;

    if vert != 0 {
        si.nPage = TREEVIEW_GetVisibleCount(infoPtr) as UINT;
        if si.nPage != 0 && !(*infoPtr).firstVisible.is_null() {
            si.nPos = (*(*infoPtr).firstVisible).visibleOrder;
            si.nMax = (*infoPtr).maxVisibleOrder - 1;

            SetScrollInfo(
                hwnd,
                (SB_VERT) as i32,
                core::ptr::addr_of_mut!(si) as LPCSCROLLINFO,
                TRUE,
            );

            if ((*infoPtr).uInternalStatus & (TV_VSCROLL as u32)) == 0 {
                ShowScrollBar(hwnd, (SB_VERT) as i32, TRUE);
            }
            (*infoPtr).uInternalStatus |= (TV_VSCROLL as u32);
        } else {
            if ((*infoPtr).uInternalStatus & (TV_VSCROLL as u32)) != 0 {
                ShowScrollBar(hwnd, (SB_VERT) as i32, FALSE);
            }
            (*infoPtr).uInternalStatus &= (!TV_VSCROLL) as u32;
        }
    } else {
        if ((*infoPtr).uInternalStatus & (TV_VSCROLL as u32)) != 0 {
            ShowScrollBar(hwnd, (SB_VERT) as i32, FALSE);
        }
        (*infoPtr).uInternalStatus &= (!TV_VSCROLL) as u32;
    }

    if horz != 0 {
        si.nPage = (*infoPtr).clientWidth as UINT;
        si.nPos = (*infoPtr).scrollX;
        si.nMax = (*infoPtr).treeWidth - 1;

        if (si.nPos as UINT)
            > (si.nMax as UINT).wrapping_sub(max(
                si.nPage.wrapping_sub(1),
                0,
            ))
        {
            si.nPos = (si.nMax as UINT).wrapping_sub(max(
                si.nPage.wrapping_sub(1),
                0,
            )) as i32;
            scrollX = si.nPos;
        }

        if ((*infoPtr).uInternalStatus & (TV_HSCROLL as u32)) == 0 {
            ShowScrollBar(hwnd, (SB_HORZ) as i32, TRUE);
        }
        (*infoPtr).uInternalStatus |= (TV_HSCROLL as u32);

        SetScrollInfo(
            hwnd,
            (SB_HORZ) as i32,
            core::ptr::addr_of_mut!(si) as LPCSCROLLINFO,
            TRUE,
        );
        TREEVIEW_HScroll(
            infoPtr,
            MAKEWPARAM!(SB_THUMBPOSITION as WORD, scrollX as WORD) as WPARAM,
        );
    } else {
        if ((*infoPtr).uInternalStatus & (TV_HSCROLL as u32)) != 0 {
            ShowScrollBar(hwnd, (SB_HORZ) as i32, FALSE);
        }
        (*infoPtr).uInternalStatus &= (!TV_HSCROLL) as u32;

        scrollX = 0;
        if (*infoPtr).scrollX != 0 {
            TREEVIEW_HScroll(
                infoPtr,
                MAKEWPARAM!(SB_THUMBPOSITION as WORD, scrollX as WORD) as WPARAM,
            );
        }
    }

    if horz == 0 {
        (*infoPtr).uInternalStatus &= (!TV_HSCROLL) as u32;
    }
}

unsafe extern "C" fn TREEVIEW_FillBkgnd(infoPtr: *const TREEVIEW_INFO, hdc: HDC, rc: *const RECT) {
    let mut hBrush: HBRUSH;
    let clrBk: COLORREF = GETBKCOLOR!((*infoPtr).clrBk);
    hBrush = CreateSolidBrush(clrBk);
    FillRect(hdc, rc, hBrush);
    DeleteObject((hBrush).cast());
}

unsafe extern "C" fn TREEVIEW_EraseBackground(infoPtr: *const TREEVIEW_INFO, hdc: HDC) -> LRESULT {
    let mut rect: RECT = core::mem::zeroed();

    GetClientRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(rect));
    TREEVIEW_FillBkgnd(infoPtr, hdc, core::ptr::addr_of_mut!(rect));

    return 1;
}

unsafe extern "C" fn TREEVIEW_Refresh(
    mut infoPtr: *mut TREEVIEW_INFO,
    mut hdc: HDC,
    mut rc: *const RECT,
) {
    let mut hwnd: HWND = (*infoPtr).hwnd;
    let mut rect: RECT = *rc;
    let mut item: *mut TREEVIEW_ITEM = ::core::ptr::null_mut::<TREEVIEW_ITEM>();
    if (*infoPtr).clientHeight == 0
        || (*infoPtr).clientWidth == 0
    {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/treeview.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TREEVIEW_Refresh\0" as *const u8 as *const ::core::ffi::c_char,
                2920 as ::core::ffi::c_int,
                b"empty window\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return;
    }
    (*infoPtr).cdmode =
        TREEVIEW_SendCustomDrawNotify(infoPtr, CDDS_PREPAINT as DWORD, hdc, rect) as INT;
    if (*infoPtr).cdmode == (CDRF_SKIPDEFAULT as i32) {
        ReleaseDC(hwnd, hdc);
        return;
    }
    item = (*(*infoPtr).root).firstChild as *mut TREEVIEW_ITEM;
    while !item.is_null() {
        if ISVISIBLE!(item) {
            if (*item).rect.top > rect.bottom {
                break;
            }
            if !((*item).rect.bottom < rect.top) {
                TREEVIEW_DrawItem(infoPtr, hdc, item);
            }
        }
        item = TREEVIEW_GetNextListItem(infoPtr, item);
    }
    if ((*infoPtr).cdmode & (CDRF_NOTIFYPOSTPAINT as i32)) != 0 {
        (*infoPtr).cdmode =
            TREEVIEW_SendCustomDrawNotify(infoPtr, CDDS_POSTPAINT as DWORD, hdc, rect) as INT;
    }
}

unsafe extern "C" fn TREEVIEW_InvalidateItem(infoPtr: *const TREEVIEW_INFO, item: *const TREEVIEW_ITEM) {
    if !item.is_null() {
        InvalidateRect((*infoPtr).hwnd, core::ptr::addr_of!((*item).rect), TRUE);
    }
}

unsafe extern "C" fn TREEVIEW_Invalidate(infoPtr: *const TREEVIEW_INFO, item: *const TREEVIEW_ITEM) {
    if !item.is_null() {
        InvalidateRect((*infoPtr).hwnd, core::ptr::addr_of!((*item).rect), TRUE);
    } else {
        InvalidateRect((*infoPtr).hwnd, core::ptr::null(), TRUE);
    }
}

unsafe extern "C" fn TREEVIEW_InitCheckboxes(infoPtr: *mut TREEVIEW_INFO) {
    let mut rc: RECT = core::mem::zeroed::<RECT>();
    let mut hbm: HBITMAP;
    let mut hbmOld: HBITMAP;
    let mut hdc: HDC;
    let mut hdcScreen: HDC;
    let mut nIndex: i32;

    (*infoPtr).himlState = ImageList_Create(16, 16, ILC_COLOR | ILC_MASK, 3, 0);

    hdcScreen = GetDC(core::ptr::null_mut());

    hdc = CreateCompatibleDC(hdcScreen);
    hbm = CreateCompatibleBitmap(hdcScreen, 48, 16);
    hbmOld = SelectObject(hdc, hbm as HGDIOBJ) as HBITMAP;

    SetRect(core::ptr::addr_of_mut!(rc), 0, 0, 48, 16);
    FillRect(hdc, core::ptr::addr_of_mut!(rc) as LPCRECT, (COLOR_WINDOW + 1) as HBRUSH);

    SetRect(core::ptr::addr_of_mut!(rc), 18, 2, 30, 14);
    DrawFrameControl(hdc, core::ptr::addr_of_mut!(rc), DFC_BUTTON,
                     DFCS_BUTTONCHECK | DFCS_FLAT);

    SetRect(core::ptr::addr_of_mut!(rc), 34, 2, 46, 14);
    DrawFrameControl(hdc, core::ptr::addr_of_mut!(rc), DFC_BUTTON,
                     DFCS_BUTTONCHECK | DFCS_FLAT | DFCS_CHECKED);

    SelectObject(hdc, hbmOld as HGDIOBJ);
    nIndex = ImageList_AddMasked((*infoPtr).himlState, hbm,
                                 comctl32_color.clrWindow);

    DeleteObject(hbm as HGDIOBJ);
    DeleteDC(hdc);
    ReleaseDC(core::ptr::null_mut(), hdcScreen);

    (*infoPtr).stateImageWidth = 16;
    (*infoPtr).stateImageHeight = 16;
}

unsafe extern "C" fn TREEVIEW_ResetImageStateIndex(infoPtr: *mut TREEVIEW_INFO, item: *mut TREEVIEW_ITEM) {
    let mut child: *mut TREEVIEW_ITEM = (*item).firstChild as *mut TREEVIEW_ITEM;
    (*item).state &= !TVIS_STATEIMAGEMASK as UINT;
    (*item).state |= (INDEXTOSTATEIMAGEMASK!(1) as UINT);
    while !child.is_null() {
        let mut next: *mut TREEVIEW_ITEM = (*child).nextSibling as *mut TREEVIEW_ITEM;
        TREEVIEW_ResetImageStateIndex(infoPtr, child);
        child = next;
    }
}

unsafe extern "C" fn TREEVIEW_Paint(infoPtr: *mut TREEVIEW_INFO, hdc_ref: HDC) -> LRESULT {
    let mut hdc: HDC = core::ptr::null_mut();
    let mut ps: PAINTSTRUCT = core::mem::zeroed::<PAINTSTRUCT>();
    let mut rc: RECT = core::mem::zeroed::<RECT>();

    if (((*infoPtr).dwStyle & (TVS_CHECKBOXES as DWORD)) != 0) && (*infoPtr).himlState.is_null() {
        TREEVIEW_InitCheckboxes(infoPtr);
        TREEVIEW_ResetImageStateIndex(infoPtr, (*infoPtr).root as *mut TREEVIEW_ITEM);

        TREEVIEW_EndEditLabelNow(infoPtr, TRUE);
        TREEVIEW_UpdateSubTree(infoPtr, (*infoPtr).root as *mut TREEVIEW_ITEM);
        TREEVIEW_UpdateScrollBars(infoPtr);
        TREEVIEW_Invalidate(infoPtr, core::ptr::null::<TREEVIEW_ITEM>());
    }

    if !hdc_ref.is_null() {
        hdc = hdc_ref;
        GetClientRect((*infoPtr).hwnd, &raw mut rc);
        TREEVIEW_FillBkgnd(infoPtr, hdc, &raw mut rc);
    } else {
        hdc = BeginPaint((*infoPtr).hwnd, &raw mut ps);
        rc = ps.rcPaint;
        if ps.fErase != 0 {
            TREEVIEW_FillBkgnd(infoPtr, hdc, &raw mut rc);
        }
    }

    if (*infoPtr).bRedraw != 0 {
        TREEVIEW_Refresh(infoPtr, hdc, &raw mut rc);
    }

    if hdc_ref.is_null() {
        EndPaint((*infoPtr).hwnd, &raw mut ps);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TREEVIEW_PrintClient(infoPtr: *mut TREEVIEW_INFO, hdc: HDC, options: DWORD) -> LRESULT {
    if (options & (PRF_CHECKVISIBLE as DWORD)) != 0 && IsWindowVisible((*infoPtr).hwnd) == 0 {
        return 0 as LRESULT;
    }

    if (options & (PRF_ERASEBKGND as DWORD)) != 0 {
        TREEVIEW_EraseBackground(infoPtr, hdc);
    }

    if (options & (PRF_CLIENT as DWORD)) != 0 {
        let mut rc: RECT = core::mem::zeroed();
        GetClientRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(rc));
        TREEVIEW_Refresh(infoPtr, hdc, core::ptr::addr_of_mut!(rc));
    }

    return 0 as LRESULT;
}

unsafe extern "system" fn TREEVIEW_CallBackCompare(first: *const TREEVIEW_ITEM, second: *const TREEVIEW_ITEM, pCallBackSort: *const TVSORTCB) -> INT {
    return (*pCallBackSort).lpfnCompare.unwrap_unchecked()((*first).lParam, (*second).lParam, (*pCallBackSort).lParam) as INT;
}
unsafe extern "system" fn TREEVIEW_SortOnName(
    mut first: *mut TREEVIEW_ITEM,
    mut second: *mut TREEVIEW_ITEM,
    mut infoPtr: *const TREEVIEW_INFO,
) -> INT {
    TREEVIEW_UpdateDispInfo(infoPtr, first, TVIF_TEXT);
    TREEVIEW_UpdateDispInfo(infoPtr, second, TVIF_TEXT);

    if !(*first).pszText.is_null() && !(*second).pszText.is_null() {
        return lstrcmpiW((*first).pszText as LPCWSTR, (*second).pszText as LPCWSTR) as INT;
    } else if !(*first).pszText.is_null() {
        return -1;
    } else if !(*second).pszText.is_null() {
        return 1;
    } else {
        return 0;
    }
}

unsafe extern "C" fn TREEVIEW_CountChildren(item: *const TREEVIEW_ITEM) -> INT {
    let mut cChildren: INT = 0;
    let mut hti: HTREEITEM;

    hti = (*item).firstChild;
    while !hti.is_null() {
        cChildren += 1;
        hti = (*hti).nextSibling;
    }

    return cChildren;
}

unsafe extern "C" fn TREEVIEW_BuildChildDPA(item: *const TREEVIEW_ITEM) -> HDPA {
    let mut child: HTREEITEM = core::ptr::null_mut();

    let list: HDPA = DPA_Create(8);
    if list == core::ptr::null_mut() {
        return core::ptr::null_mut();
    }

    child = (*item).firstChild;
    while !child.is_null() {
        if DPA_InsertPtr(list, (INT_MAX) as INT, child as *mut ::core::ffi::c_void) == -1 {
            DPA_Destroy(list);
            return core::ptr::null_mut();
        }
        child = (*child).nextSibling;
    }

    return list;
}

unsafe extern "C" fn TREEVIEW_Sort(
    mut infoPtr: *mut TREEVIEW_INFO,
    mut parent: HTREEITEM,
    mut pSort: LPTVSORTCB,
) -> LRESULT {
    let mut cChildren: INT = 0;
    let mut pfnCompare: PFNDPACOMPARE = None;
    let mut lpCompare: LPARAM = 0;

    /* undocumented feature: TVI_ROOT or NULL means `sort the whole tree' */
    if parent == TVI_ROOT || parent.is_null() {
        parent = (*infoPtr).root;
    }

    /* Check for a valid handle to the parent item */
    if TREEVIEW_ValidItem(infoPtr, parent) == 0 {
        return FALSE as LRESULT;
    }

    if !pSort.is_null() {
        pfnCompare = ::core::mem::transmute::<
            Option<
                unsafe extern "system" fn(
                    *const TREEVIEW_ITEM,
                    *const TREEVIEW_ITEM,
                    *const TVSORTCB,
                ) -> INT,
            >,
            PFNDPACOMPARE,
        >(Some(
            TREEVIEW_CallBackCompare,
        ));
        lpCompare = pSort as LPARAM;
    } else {
        pfnCompare = ::core::mem::transmute::<
            Option<
                unsafe extern "system" fn(
                    *mut TREEVIEW_ITEM,
                    *mut TREEVIEW_ITEM,
                    *const TREEVIEW_INFO,
                ) -> INT,
            >,
            PFNDPACOMPARE,
        >(Some(
            TREEVIEW_SortOnName,
        ));
        lpCompare = infoPtr as LPARAM;
    }

    cChildren = TREEVIEW_CountChildren(parent as *const TREEVIEW_ITEM);

    /* Make sure there is something to sort */
    if cChildren > 1 {
        /* TREEVIEW_ITEM rechaining */
        let mut count: INT = 0;
        let mut item: HTREEITEM = core::ptr::null_mut();
        let mut nextItem: HTREEITEM = core::ptr::null_mut();
        let mut prevItem: HTREEITEM = core::ptr::null_mut();

        let mut sortList: HDPA = TREEVIEW_BuildChildDPA(parent as *const TREEVIEW_ITEM);

        if sortList.is_null() {
            return FALSE as LRESULT;
        }

        /* let DPA sort the list */
        DPA_Sort(sortList, pfnCompare, lpCompare);

        /* The order of DPA entries has been changed, so fixup the
         * nextSibling and prevSibling pointers. */
        let fresh16 = count;
        count = count + 1;
        item = DPA_GetPtr(sortList, fresh16) as HTREEITEM;
        loop {
            let fresh17 = count;
            count = count + 1;
            nextItem = DPA_GetPtr(sortList, fresh17) as HTREEITEM;
            if nextItem.is_null() {
                break;
            }

            /* link the two current item together */
            (*item).nextSibling = nextItem;
            (*nextItem).prevSibling = item;

            if prevItem.is_null() {
                /* this is the first item, update the parent */
                (*parent).firstChild = item;
                (*item).prevSibling = ::core::ptr::null_mut::<_TREEITEM>();
            } else {
                /* fix the back chaining */
                (*item).prevSibling = prevItem;
            }

            /* get ready for the next one */
            prevItem = item;
            item = nextItem;
        }

        /* the last item is pointed to by item and never has a sibling */
        (*item).nextSibling = ::core::ptr::null_mut::<_TREEITEM>();
        (*parent).lastChild = item;

        DPA_Destroy(sortList);

        TREEVIEW_VerifyTree(infoPtr);

        if (*parent).state & TVIS_EXPANDED as UINT != 0 {
            let mut visOrder: ::core::ffi::c_int =
                (*(*infoPtr).firstVisible).visibleOrder as ::core::ffi::c_int;

            if parent == (*infoPtr).root {
                TREEVIEW_RecalculateVisibleOrder(infoPtr, ::core::ptr::null_mut::<TREEVIEW_ITEM>());
            } else {
                TREEVIEW_RecalculateVisibleOrder(infoPtr, parent as *mut TREEVIEW_ITEM);
            }

            if TREEVIEW_IsChildOf(
                parent as *const TREEVIEW_ITEM,
                (*infoPtr).firstVisible as *const TREEVIEW_ITEM,
            ) != 0
            {
                let mut item_0: *mut TREEVIEW_ITEM = ::core::ptr::null_mut::<TREEVIEW_ITEM>();
                item_0 = (*(*infoPtr).root).firstChild as *mut TREEVIEW_ITEM;
                while !item_0.is_null() {
                    if (*item_0).visibleOrder == visOrder {
                        break;
                    }
                    item_0 = TREEVIEW_GetNextListItem(infoPtr, item_0);
                }

                if item_0.is_null() {
                    item_0 = (*parent).firstChild as *mut TREEVIEW_ITEM;
                }
                TREEVIEW_SetFirstVisible(infoPtr, item_0, FALSE);
            }

            TREEVIEW_Invalidate(infoPtr, ::core::ptr::null::<TREEVIEW_ITEM>());
        }

        return TRUE as LRESULT;
    }
    return FALSE as LRESULT;
}

unsafe extern "C" fn TREEVIEW_SortChildrenCB(infoPtr: *mut TREEVIEW_INFO, pSort: LPTVSORTCB) -> LRESULT {
    return TREEVIEW_Sort(infoPtr, ((*pSort).hParent).cast(), pSort);
}

unsafe extern "C" fn TREEVIEW_SortChildren(infoPtr: *mut TREEVIEW_INFO, lParam: LPARAM) -> LRESULT {
    return TREEVIEW_Sort(infoPtr, lParam as HTREEITEM, core::ptr::null_mut());
}

unsafe extern "C" fn TREEVIEW_SendExpanding(
    infoPtr: *const TREEVIEW_INFO,
    item: *mut TREEVIEW_ITEM,
    action: UINT,
) -> BOOL {
    return (TREEVIEW_SendTreeviewNotify(
        infoPtr,
        TVN_ITEMEXPANDINGW as UINT,
        action,
        (TVIF_HANDLE | TVIF_STATE | TVIF_PARAM
            | TVIF_IMAGE | TVIF_SELECTEDIMAGE) as UINT,
        core::ptr::null_mut::<_TREEITEM>(),
        item as HTREEITEM,
    ) == 0) as i32;
}

unsafe extern "C" fn TREEVIEW_SendExpanded(infoPtr: *const TREEVIEW_INFO, item: *mut TREEVIEW_ITEM, action: UINT) {
    TREEVIEW_SendTreeviewNotify(
        infoPtr,
        TVN_ITEMEXPANDEDW as UINT,
        action,
        (TVIF_HANDLE | TVIF_STATE | TVIF_PARAM
            | TVIF_IMAGE | TVIF_SELECTEDIMAGE) as UINT,
        0 as HTREEITEM,
        item as HTREEITEM,
    );
}

unsafe extern "C" fn TREEVIEW_Collapse(
    mut infoPtr: *mut TREEVIEW_INFO,
    mut item: *mut TREEVIEW_ITEM,
    mut bRemoveChildren: BOOL,
    mut bUser: BOOL,
) -> BOOL {
    let mut action: UINT = ((TVE_COLLAPSE as UINT)
        | (if bRemoveChildren != 0 {
            TVE_COLLAPSERESET as UINT
        } else {
            0 as UINT
        })) as UINT;
    let mut bSetSelection: BOOL = 0;
    let mut bSetFirstVisible: BOOL = 0;
    let mut scrollRect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut scrollDist: LONG = 0 as LONG;
    let mut nextItem: *mut TREEVIEW_ITEM = core::ptr::null_mut::<TREEVIEW_ITEM>();
    let mut tmpItem: *mut TREEVIEW_ITEM = core::ptr::null_mut::<TREEVIEW_ITEM>();
    let mut wasExpanded: BOOL = 0;

    if TREEVIEW_HasChildren(infoPtr, item) == 0 {
        return FALSE as BOOL;
    }

    if bUser != 0 {
        TREEVIEW_SendExpanding(infoPtr, item, action);
    }

    if (*item).firstChild.is_null() {
        return FALSE as BOOL;
    }

    wasExpanded = (((*item).state & ((TVIS_EXPANDED) as UINT))
        != ((0) as UINT)) as ::core::ffi::c_int as BOOL;
    (*item).state &= !((TVIS_EXPANDED) as UINT);

    if wasExpanded != 0 && bUser != 0 {
        TREEVIEW_SendExpanded(infoPtr, item, action);
    }

    bSetSelection = (!(*infoPtr).selectedItem.is_null()
        && TREEVIEW_IsChildOf(item, (*infoPtr).selectedItem as *const TREEVIEW_ITEM) != 0)
        as ::core::ffi::c_int as BOOL;

    bSetFirstVisible = (!(*infoPtr).firstVisible.is_null()
        && TREEVIEW_IsChildOf(item, (*infoPtr).firstVisible as *const TREEVIEW_ITEM) != 0)
        as ::core::ffi::c_int as BOOL;

    tmpItem = item;
    while !tmpItem.is_null() {
        if !(*tmpItem).nextSibling.is_null() {
            nextItem = (*tmpItem).nextSibling as *mut TREEVIEW_ITEM;
            break;
        } else {
            tmpItem = (*tmpItem).parent as *mut TREEVIEW_ITEM;
        }
    }

    if !nextItem.is_null() {
        scrollDist = (*nextItem).rect.top;
    }

    if bRemoveChildren != 0 {
        let mut old_cChildren: INT = (*item).cChildren as INT;
        (*item).state &= !((TVIS_EXPANDEDONCE) as UINT);
        TREEVIEW_RemoveAllChildren(infoPtr, item);
        (*item).cChildren = old_cChildren as ::core::ffi::c_int;
    }

    if wasExpanded == 0 {
        return FALSE as BOOL;
    }

    if !(*item).firstChild.is_null() {
        let mut i: *mut TREEVIEW_ITEM = core::ptr::null_mut::<TREEVIEW_ITEM>();
        let mut sibling: *mut TREEVIEW_ITEM = core::ptr::null_mut::<TREEVIEW_ITEM>();

        sibling = TREEVIEW_GetNextListItem(infoPtr, item);

        i = (*item).firstChild as *mut TREEVIEW_ITEM;
        while i != sibling {
            (*i).visibleOrder = -(1 as LONG);
            i = TREEVIEW_GetNextListItem(infoPtr, i);
        }
    }

    TREEVIEW_RecalculateVisibleOrder(infoPtr, item);

    if !nextItem.is_null() {
        scrollDist = -(scrollDist - (*nextItem).rect.top);
    }

    if bSetSelection != 0 {
        if TREEVIEW_ValidItem(infoPtr, (*infoPtr).selectedItem) != 0 {
            (*(*infoPtr).selectedItem).state &= !((TVIS_SELECTED) as UINT);
        }
        (*item).state |= (TVIS_SELECTED) as UINT;
        (*infoPtr).selectedItem = item as HTREEITEM;
    }

    TREEVIEW_UpdateScrollBars(infoPtr);

    scrollRect.left = 0 as LONG;
    scrollRect.right = (*infoPtr).clientWidth;
    scrollRect.bottom = (*infoPtr).clientHeight;

    if !nextItem.is_null() {
        scrollRect.top = (*nextItem).rect.top;

        ScrollWindowEx(
            (*infoPtr).hwnd,
            0 as ::core::ffi::c_int,
            scrollDist as ::core::ffi::c_int,
            &raw mut scrollRect as LPCRECT,
            &raw mut scrollRect as LPCRECT,
            core::ptr::null_mut::<HRGN__>(),
            core::ptr::null_mut::<RECT>(),
            (SW_ERASE | SW_INVALIDATE) as UINT,
        );
        TREEVIEW_Invalidate(infoPtr, item);
    } else {
        scrollRect.top = (*item).rect.top;
        InvalidateRect(
            (*infoPtr).hwnd,
            &raw mut scrollRect as LPCRECT,
            TRUE as BOOL,
        );
    }

    TREEVIEW_SetFirstVisible(
        infoPtr,
        if bSetFirstVisible != 0 {
            item
        } else {
            (*infoPtr).firstVisible as *mut TREEVIEW_ITEM
        },
        TRUE as BOOL,
    );

    return wasExpanded;
}

unsafe extern "C" fn TREEVIEW_Expand(
    infoPtr: *mut TREEVIEW_INFO,
    item: *mut TREEVIEW_ITEM,
    partial: BOOL,
    user: BOOL,
) -> BOOL {
    let mut scrollDist: LONG = 0;
    let mut orgNextTop: LONG = 0 as LONG;
    let mut scrollRect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut nextItem: *mut TREEVIEW_ITEM = core::ptr::null_mut();
    let mut tmpItem: *mut TREEVIEW_ITEM = core::ptr::null_mut();
    let mut sendsNotifications: BOOL = 0;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TREEVIEW_Expand\0" as *const u8 as *const ::core::ffi::c_char,
            3463 as ::core::ffi::c_int,
            b"(%p, %p, partial=%d, %d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            infoPtr,
            item,
            partial,
            user,
        );
    }

    if TREEVIEW_HasChildren(infoPtr, item) == 0 {
        return FALSE;
    }

    tmpItem = item;
    nextItem = core::ptr::null_mut();
    while !tmpItem.is_null() {
        if !(*tmpItem).nextSibling.is_null() {
            nextItem = (*tmpItem).nextSibling as *mut TREEVIEW_ITEM;
            break;
        } else {
            tmpItem = (*tmpItem).parent as *mut TREEVIEW_ITEM;
        }
    }

    if !nextItem.is_null() {
        orgNextTop = (*nextItem).rect.top;
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
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TREEVIEW_Expand\0" as *const u8 as *const ::core::ffi::c_char,
            3482 as ::core::ffi::c_int,
            b"TVE_EXPAND %p %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            item,
            TREEVIEW_ItemName(item),
        );
    }

    sendsNotifications = (user != 0
        || ((*item).cChildren != (0 as ::core::ffi::c_int)
            && ((*item).state & (TVIS_EXPANDEDONCE as UINT)) == 0))
        as ::core::ffi::c_int as BOOL;

    if sendsNotifications != 0 {
        if TREEVIEW_SendExpanding(infoPtr, item, TVE_EXPAND as UINT) == 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/treeview.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TREEVIEW_Expand\0" as *const u8 as *const ::core::ffi::c_char,
                    3490 as ::core::ffi::c_int,
                    b"  TVN_ITEMEXPANDING returned TRUE, exiting...\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            return FALSE;
        }
    }

    if (*item).firstChild.is_null() {
        return FALSE;
    }

    (*item).state |= TVIS_EXPANDED as UINT;

    if partial != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl_2,
                __dbch_2,
                b"dll/win32/comctl32/treeview.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TREEVIEW_Expand\0" as *const u8 as *const ::core::ffi::c_char,
                3500 as ::core::ffi::c_int,
                b"TVE_EXPANDPARTIAL not implemented\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    }

    if ISVISIBLE!(item) != false {
        TREEVIEW_RecalculateVisibleOrder(infoPtr, item);
        TREEVIEW_UpdateSubTree(infoPtr, item);
        TREEVIEW_UpdateScrollBars(infoPtr);

        scrollRect.left = 0 as LONG;
        scrollRect.bottom = (*infoPtr).treeHeight;
        scrollRect.right = (*infoPtr).clientWidth;
        if !nextItem.is_null() {
            scrollDist = (*nextItem).rect.top - orgNextTop;
            scrollRect.top = orgNextTop;

            ScrollWindowEx(
                (*infoPtr).hwnd,
                0 as ::core::ffi::c_int,
                scrollDist as ::core::ffi::c_int,
                core::ptr::addr_of_mut!(scrollRect) as LPCRECT,
                core::ptr::null(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                (SW_ERASE | SW_INVALIDATE) as UINT,
            );
            TREEVIEW_Invalidate(infoPtr, item);
        } else {
            scrollRect.top = (*item).rect.top;
            InvalidateRect(
                (*infoPtr).hwnd,
                core::ptr::addr_of_mut!(scrollRect) as LPCRECT,
                FALSE as BOOL,
            );
        }

        /* Scroll up so that as many children as possible are visible.
        * This fails when expanding causes an HScroll bar to appear, but we
        * don't know that yet, so the last item is obscured. */
        if !(*item).firstChild.is_null() {
            let mut nChildren: ::core::ffi::c_int = (*(*item).lastChild).visibleOrder
                as ::core::ffi::c_int
                - (*(*item).firstChild).visibleOrder as ::core::ffi::c_int
                + (1 as ::core::ffi::c_int);

            let mut visible_pos: ::core::ffi::c_int = (*item).visibleOrder as ::core::ffi::c_int
                - (*(*infoPtr).firstVisible).visibleOrder as ::core::ffi::c_int;

            let mut rows_below: ::core::ffi::c_int = (TREEVIEW_GetVisibleCount(infoPtr)
                - visible_pos as LRESULT
                - (1 as LRESULT))
                as ::core::ffi::c_int;

            if visible_pos > (0 as ::core::ffi::c_int)
                && nChildren > rows_below
            {
                let mut scroll: ::core::ffi::c_int = nChildren - rows_below;

                if scroll > visible_pos {
                    scroll = visible_pos;
                }

                if scroll > (0 as ::core::ffi::c_int) {
                    let mut newFirstVisible: *mut TREEVIEW_ITEM = TREEVIEW_GetListItem(
                        infoPtr,
                        (*infoPtr).firstVisible as *mut TREEVIEW_ITEM,
                        scroll as LONG,
                    );

                    TREEVIEW_SetFirstVisible(
                        infoPtr,
                        newFirstVisible,
                        TRUE as BOOL,
                    );
                }
            }
        }
    }

    if sendsNotifications != 0 {
        TREEVIEW_SendExpanded(infoPtr, item, TVE_EXPAND as UINT);
        (*item).state |= TVIS_EXPANDEDONCE as UINT;
    }

    return TRUE;
}

unsafe extern "C" fn TREEVIEW_SingleExpand(
    mut infoPtr: *mut TREEVIEW_INFO,
    mut selection: HTREEITEM,
    mut item: HTREEITEM,
) {
    let mut prev: *mut TREEVIEW_ITEM = ::core::ptr::null_mut::<TREEVIEW_ITEM>();
    let mut curr: *mut TREEVIEW_ITEM = ::core::ptr::null_mut::<TREEVIEW_ITEM>();
    if ((*infoPtr).dwStyle & TVS_SINGLEEXPAND) == 0
        || !(*infoPtr).hwndEdit.is_null()
        || item.is_null()
    {
        return;
    }
    TREEVIEW_SendTreeviewNotify(
        infoPtr,
        (TVN_SINGLEEXPAND) as UINT,
        TVC_UNKNOWN,
        TVIF_HANDLE | TVIF_PARAM,
        item,
        core::ptr::null_mut(),
    );
    /*
     * Close the previous item and its ancestors as long as they are not
     * ancestors of the current item
     */
    prev = selection as *mut TREEVIEW_ITEM;
    'finish: {
        while !prev.is_null() && TREEVIEW_ValidItem(infoPtr, prev as HTREEITEM) != 0 {
            curr = item as *mut TREEVIEW_ITEM;
            while !curr.is_null() && TREEVIEW_ValidItem(infoPtr, curr as HTREEITEM) != 0 {
                if curr == prev {
                    break 'finish;
                }
                curr = (*curr).parent as *mut TREEVIEW_ITEM;
            }
            TREEVIEW_Collapse(infoPtr, prev, FALSE, TRUE);
            prev = (*prev).parent as *mut TREEVIEW_ITEM;
        }
    }
    /*
     * Expand the current item
     */
    TREEVIEW_Expand(
        infoPtr,
        item as *mut TREEVIEW_ITEM,
        FALSE,
        TRUE,
    );
}

unsafe extern "C" fn TREEVIEW_Toggle(infoPtr: *mut TREEVIEW_INFO, item: *mut TREEVIEW_ITEM, user: BOOL) -> BOOL {
    if ((*item).state & TVIS_EXPANDED) != 0 {
        return TREEVIEW_Collapse(infoPtr, item, FALSE, user);
    } else {
        return TREEVIEW_Expand(infoPtr, item, FALSE, user);
    }
}

unsafe extern "C" fn TREEVIEW_ExpandAll(infoPtr: *mut TREEVIEW_INFO, mut item: *mut TREEVIEW_ITEM) {
    TREEVIEW_Expand(infoPtr, item, FALSE, TRUE);
    item = (*item).firstChild as *mut TREEVIEW_ITEM;
    while !item.is_null() {
        if TREEVIEW_HasChildren(infoPtr, item) != 0 {
            TREEVIEW_ExpandAll(infoPtr, item);
        }
        item = (*item).nextSibling as *mut TREEVIEW_ITEM;
    }
}

unsafe extern "C" fn TREEVIEW_ExpandMsg(infoPtr: *mut TREEVIEW_INFO, flag: UINT, item: HTREEITEM) -> LRESULT {
    if TREEVIEW_ValidItem(infoPtr, item) == 0 {
        return 0 as LRESULT;
    }

    match flag & TVE_TOGGLE as UINT {
        TVE_COLLAPSE => {
            return TREEVIEW_Collapse(
                infoPtr,
                item as *mut TREEVIEW_ITEM,
                (flag & TVE_COLLAPSERESET as UINT) as BOOL,
                FALSE as BOOL,
            ) as LRESULT;
        }
        TVE_EXPAND => {
            return TREEVIEW_Expand(
                infoPtr,
                item as *mut TREEVIEW_ITEM,
                (flag & TVE_EXPANDPARTIAL as UINT) as BOOL,
                FALSE as BOOL,
            ) as LRESULT;
        }
        TVE_TOGGLE => {
            return TREEVIEW_Toggle(
                infoPtr,
                item as *mut TREEVIEW_ITEM,
                FALSE as BOOL,
            ) as LRESULT;
        }
        _ => {
            return 0 as LRESULT;
        }
    }
}

unsafe extern "C" fn TREEVIEW_HitTestPoint(
    infoPtr: *const TREEVIEW_INFO,
    pt: POINT,
) -> *mut TREEVIEW_ITEM {
    let mut item: *mut TREEVIEW_ITEM;
    let mut row: LONG;

    if (*infoPtr).firstVisible.is_null() {
        return core::ptr::null_mut();
    }

    row = (pt.y / (*infoPtr).uItemHeight as LONG) + (*(*infoPtr).firstVisible).visibleOrder;

    item = (*infoPtr).firstVisible as *mut TREEVIEW_ITEM;
    while !item.is_null() {
        if row >= (*item).visibleOrder
            && row < (*item).visibleOrder + (*item).iIntegral
        {
            break;
        }
        item = TREEVIEW_GetNextListItem(infoPtr, item);
    }

    return item;
}

unsafe extern "C" fn TREEVIEW_HitTest(
    infoPtr: *const TREEVIEW_INFO,
    lpht: LPTVHITTESTINFO,
) -> *mut TREEVIEW_ITEM {
    let mut item: *mut TREEVIEW_ITEM = core::ptr::null_mut::<TREEVIEW_ITEM>();
    let mut rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut status: UINT = 0;
    let mut x: LONG = 0;
    let mut y: LONG = 0;

    (*lpht).hItem = core::ptr::null_mut();
    GetClientRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(rect));
    status = 0;
    x = (*lpht).pt.x;
    y = (*lpht).pt.y;

    if x < rect.left {
        status |= TVHT_TOLEFT;
    } else if x > rect.right {
        status |= TVHT_TORIGHT;
    }

    if y < rect.top {
        status |= TVHT_ABOVE;
    } else if y > rect.bottom {
        status |= TVHT_BELOW;
    }

    if status != 0 {
        (*lpht).flags = status;
        return core::ptr::null_mut::<TREEVIEW_ITEM>();
    }

    item = TREEVIEW_HitTestPoint(infoPtr, (*lpht).pt);
    if item.is_null() {
        (*lpht).flags = TVHT_NOWHERE;
        return core::ptr::null_mut::<TREEVIEW_ITEM>();
    }

    if (*item).textWidth == 0 {
        TREEVIEW_ComputeTextWidth(infoPtr, item, core::ptr::null_mut());
    }

    if x >= (*item).textOffset + (*item).textWidth {
        (*lpht).flags = TVHT_ONITEMRIGHT;
    } else if x >= (*item).textOffset {
        (*lpht).flags = TVHT_ONITEMLABEL;
    } else if x >= (*item).imageOffset {
        (*lpht).flags = TVHT_ONITEMICON;
    } else if x >= (*item).stateOffset {
        (*lpht).flags = TVHT_ONITEMSTATEICON;
    } else if x >= (*item).linesOffset && ((*infoPtr).dwStyle & TVS_HASBUTTONS) != 0 {
        (*lpht).flags = TVHT_ONITEMBUTTON;
    } else {
        (*lpht).flags = TVHT_ONITEMINDENT;
    }

    (*lpht).hItem = (item as HTREEITEM).cast();

    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TREEVIEW_HitTest\0" as *const u8 as *const ::core::ffi::c_char,
            3756 as ::core::ffi::c_int,
            b"(%d,%d):result 0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*lpht).pt.x,
            (*lpht).pt.y,
            (*lpht).flags,
        );
    }

    return item;
}

unsafe extern "C" fn TREEVIEW_GetEditControl(infoPtr: *const TREEVIEW_INFO) -> LRESULT {
    return (*infoPtr).hwndEdit as LRESULT;
}

unsafe extern "system" fn TREEVIEW_Edit_SubclassProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let infoPtr: *mut TREEVIEW_INFO = TREEVIEW_GetInfoPtr(GetParent(hwnd));
    let mut bCancel: BOOL = FALSE;
    let mut rc: LRESULT = 0;

    'sw: {
        match uMsg {
            WM_PAINT => {
                // TRACE("WM_PAINT start\n");
                rc = CallWindowProcW((*infoPtr).wpEditOrig, hwnd, uMsg, wParam, lParam);
                // TRACE("WM_PAINT done\n");
                return rc;
            }
            WM_KILLFOCUS => {
                if (*infoPtr).bIgnoreEditKillFocus != 0 {
                    return TRUE as LRESULT;
                }
            }
            WM_DESTROY => {
                let editProc: WNDPROC = (*infoPtr).wpEditOrig;
                (*infoPtr).wpEditOrig = None;
                SetWindowLongPtrW(
                    hwnd,
                    GWLP_WNDPROC,
                    ::core::mem::transmute::<WNDPROC, DWORD_PTR>(editProc) as LONG_PTR,
                );
                return CallWindowProcW(editProc, hwnd, uMsg, wParam, lParam);
            }
            WM_GETDLGCODE => {
                return (DLGC_WANTARROWS | DLGC_WANTALLKEYS) as LRESULT;
            }
            WM_KEYDOWN => {
                if wParam == VK_ESCAPE as WPARAM {
                    bCancel = TRUE;
                    break 'sw;
                } else if wParam == VK_RETURN as WPARAM {
                    break 'sw;
                }
                /* fall through */
                return CallWindowProcW((*infoPtr).wpEditOrig, hwnd, uMsg, wParam, lParam);
            }
            _ => {
                return CallWindowProcW((*infoPtr).wpEditOrig, hwnd, uMsg, wParam, lParam);
            }
        }
    }

    /* Processing TVN_ENDLABELEDIT message could kill the focus       */
    /* eg. Using a messagebox                                         */

    (*infoPtr).bIgnoreEditKillFocus = TRUE;
    TREEVIEW_EndEditLabelNow(
        infoPtr,
        (bCancel != 0 || (*infoPtr).bLabelChanged == 0) as BOOL,
    );
    (*infoPtr).bIgnoreEditKillFocus = FALSE;

    return 0 as LRESULT;
}

unsafe extern "C" fn TREEVIEW_Command(infoPtr: *mut TREEVIEW_INFO, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    match (HIWORD!(wParam) as WORD as i32) as u32 {
        EN_UPDATE => {
            let mut buffer: [WCHAR; 1024] = [0; 1024];
            let mut editItem: *mut TREEVIEW_ITEM = (*infoPtr).editItem as *mut TREEVIEW_ITEM;
            let mut hdc: HDC = GetDC((*infoPtr).hwndEdit);
            let mut sz: SIZE = core::mem::zeroed();
            let mut hFont: HFONT;
            let mut hOldFont: HFONT = core::ptr::null_mut();
            if IsWindow((*infoPtr).hwndEdit) == 0 || hdc.is_null() {
                return FALSE as LRESULT;
            }
            (*infoPtr).bLabelChanged = TRUE as BOOL;
            GetWindowTextW(
                (*infoPtr).hwndEdit,
                &raw mut buffer as LPWSTR,
                ARRAY_SIZE!(buffer) as i32,
            );
            hFont = SendMessageW(
                (*infoPtr).hwndEdit,
                WM_GETFONT as UINT,
                0 as WPARAM,
                0 as LPARAM,
            ) as HFONT;
            if hFont != core::ptr::null_mut() {
                hOldFont = SelectObject(hdc, hFont as HGDIOBJ) as HFONT;
            }
            if GetTextExtentPoint32W(
                hdc,
                &raw mut buffer as *mut WCHAR as LPCWSTR,
                lstrlenW(&raw mut buffer as *mut WCHAR as LPCWSTR),
                &raw mut sz,
            ) != 0
            {
                let mut textMetric: TEXTMETRICW = core::mem::zeroed();
                GetTextMetricsW(hdc, &raw mut textMetric);
                sz.cx += textMetric.tmMaxCharWidth * 2;

                sz.cx = max(sz.cx, textMetric.tmMaxCharWidth * 3);
                sz.cx = min(
                    sz.cx,
                    (*infoPtr).clientWidth - (*editItem).textOffset + 2,
                );

                SetWindowPos(
                    (*infoPtr).hwndEdit,
                    HWND_TOP,
                    0,
                    0,
                    sz.cx,
                    (*editItem).rect.bottom - (*editItem).rect.top + 3,
                    (SWP_NOMOVE | SWP_DRAWFRAME) as UINT,
                );
            }
            if hFont != core::ptr::null_mut() {
                SelectObject(hdc, hOldFont as HGDIOBJ);
            }
            ReleaseDC((*infoPtr).hwnd, hdc);
        }
        EN_KILLFOCUS => {
            if (*infoPtr).hwndEdit != lParam as HWND {
                return FALSE as LRESULT;
            }
            TREEVIEW_EndEditLabelNow(infoPtr, FALSE as BOOL);
        }
        _ => {
            return SendMessageW(
                (*infoPtr).hwndNotify,
                WM_COMMAND as UINT,
                wParam,
                lParam,
            );
        }
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn TREEVIEW_EditLabel(
    infoPtr: *mut TREEVIEW_INFO,
    hItem: HTREEITEM,
) -> HWND {
    let hwnd: HWND = (*infoPtr).hwnd;
    let mut hwndEdit: HWND;
    let mut sz: SIZE = core::mem::zeroed();
    let hinst: HINSTANCE = GetWindowLongPtrW(hwnd, GWLP_HINSTANCE) as HINSTANCE;
    let mut hdc: HDC;
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut textMetric: TEXTMETRICW = core::mem::zeroed();

    if !((*infoPtr).dwStyle & TVS_EDITLABELS != 0) {
        return core::ptr::null_mut();
    }

    if TREEVIEW_ValidItem(infoPtr, hItem) == 0 {
        return core::ptr::null_mut();
    }

    if !(*infoPtr).hwndEdit.is_null() {
        return (*infoPtr).hwndEdit;
    }

    (*infoPtr).bLabelChanged = FALSE;

    /* make edit item visible */
    TREEVIEW_EnsureVisible(infoPtr, hItem, TRUE);

    TREEVIEW_UpdateDispInfo(infoPtr, (hItem).cast(), TVIF_TEXT);

    hdc = GetDC(hwnd);
    /* Select the font to get appropriate metric dimensions */
    if (*infoPtr).hFont != core::ptr::null_mut() {
        hOldFont = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;
    }

    /* Get string length in pixels */
    if !(*hItem).pszText.is_null() {
        GetTextExtentPoint32W(
            hdc,
            (*hItem).pszText as LPCWSTR,
            lstrlenW((*hItem).pszText as LPCWSTR),
            &mut sz,
        );
    } else {
        GetTextExtentPoint32A(
            hdc,
            b"\0" as *const u8 as LPCSTR,
            0,
            &mut sz,
        );
    }

    /* Add Extra spacing for the next character */
    GetTextMetricsW(hdc, &mut textMetric);
    sz.cx += textMetric.tmMaxCharWidth * 2;

    sz.cx = max(sz.cx, textMetric.tmMaxCharWidth * 3);
    sz.cx = min(
        sz.cx,
        (*infoPtr).clientWidth - (*hItem).textOffset + 2,
    );

    if (*infoPtr).hFont != core::ptr::null_mut() {
        SelectObject(hdc, hOldFont as HGDIOBJ);
    }

    ReleaseDC(hwnd, hdc);

    (*infoPtr).editItem = hItem;

    hwndEdit = CreateWindowExW(
        WS_EX_LEFT,
        (WC_EDITW).as_ptr(),
        core::ptr::null(),
        (WS_CHILD
            | WS_BORDER
            | ES_AUTOHSCROLL
            | WS_CLIPSIBLINGS
            | ES_WANTRETURN
            | ES_LEFT) as DWORD,
        (*hItem).textOffset - 2,
        (*hItem).rect.top - 1,
        sz.cx + 3,
        (*hItem).rect.bottom - (*hItem).rect.top + 3,
        hwnd,
        core::ptr::null_mut(),
        hinst,
        core::ptr::null_mut(),
    );
    /* FIXME: (HMENU)IDTVEDIT,pcs->hInstance,0); */

    (*infoPtr).hwndEdit = hwndEdit;

    /* Get a 2D border. */
    SetWindowLongW(
        hwndEdit,
        GWL_EXSTYLE,
        GetWindowLongW(hwndEdit, GWL_EXSTYLE) & (!(WS_EX_CLIENTEDGE)) as i32,
    );
    SetWindowLongW(
        hwndEdit,
        GWL_STYLE,
        GetWindowLongW(hwndEdit, GWL_STYLE) | (WS_BORDER) as i32,
    );

    SendMessageW(
        hwndEdit,
        WM_SETFONT,
        TREEVIEW_FontForItem(infoPtr, (hItem).cast()) as WPARAM,
        (FALSE) as LPARAM,
    );

    (*infoPtr).wpEditOrig = core::mem::transmute::<isize, WNDPROC>((SetWindowLongPtrW(
        hwndEdit,
        GWLP_WNDPROC,
        (TREEVIEW_Edit_SubclassProc as DWORD_PTR) as LONG_PTR,
    )) as _);
    SendMessageW(
        hwndEdit,
        EM_SETLIMITTEXT,
        (MAX_PATH - 1) as WPARAM,
        0,
    );
    if !(*hItem).pszText.is_null() {
        SetWindowTextW(hwndEdit, (*hItem).pszText as LPCWSTR);
    }

    if TREEVIEW_BeginLabelEditNotify(infoPtr, (hItem).cast()) != 0 {
        DestroyWindow(hwndEdit);
        (*infoPtr).hwndEdit = core::ptr::null_mut();
        (*infoPtr).editItem = core::ptr::null_mut();
        return core::ptr::null_mut();
    }

    SetFocus(hwndEdit);
    SendMessageW(
        hwndEdit,
        EM_SETSEL,
        0 as WPARAM,
        -(1) as LPARAM,
    );
    ShowWindow(hwndEdit, (SW_SHOW) as i32);

    return hwndEdit;
}

unsafe extern "C" fn TREEVIEW_EndEditLabelNow(infoPtr: *mut TREEVIEW_INFO, bCancel: BOOL) -> LRESULT {
    let editedItem: *mut TREEVIEW_ITEM = (*infoPtr).editItem as *mut TREEVIEW_ITEM;
    let mut tvdi: NMTVDISPINFOW = core::mem::zeroed();
    let mut bCommit: BOOL = 0;
    let mut tmpText: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];
    let mut newText: *mut WCHAR = core::ptr::null_mut();
    let mut iLength: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

    if IsWindow((*infoPtr).hwndEdit) == 0 {
        return FALSE as LRESULT;
    }

    tvdi.item.mask = 0 as UINT;
    tvdi.item.hItem = (editedItem as HTREEITEM).cast();
    tvdi.item.state = (*editedItem).state;
    tvdi.item.lParam = (*editedItem).lParam;

    if bCancel == 0 {
        if (*infoPtr).bNtfUnicode == 0 {
            iLength = GetWindowTextA(
                (*infoPtr).hwndEdit,
                core::ptr::addr_of_mut!(tmpText) as *mut WCHAR as LPSTR,
                ARRAY_SIZE!(tmpText) as ::core::ffi::c_int,
            );
        } else {
            iLength = GetWindowTextW(
                (*infoPtr).hwndEdit,
                core::ptr::addr_of_mut!(tmpText) as *mut WCHAR as LPWSTR,
                ARRAY_SIZE!(tmpText) as ::core::ffi::c_int,
            );
        }

        tvdi.item.mask = TVIF_TEXT as UINT;
        tvdi.item.pszText = core::ptr::addr_of_mut!(tmpText) as *mut WCHAR as LPWSTR;
        tvdi.item.cchTextMax = ARRAY_SIZE!(tmpText) as ::core::ffi::c_int;
    } else {
        tvdi.item.pszText = core::ptr::null_mut();
        tvdi.item.cchTextMax = 0 as ::core::ffi::c_int;
    }

    bCommit = TREEVIEW_SendRealNotify(
        infoPtr,
        TVN_ENDLABELEDITW as UINT,
        core::ptr::addr_of_mut!(tvdi.hdr),
    );

    if bCancel == 0 && bCommit != 0 {
        if (*infoPtr).bNtfUnicode == 0 {
            let len: DWORD = MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                tvdi.item.pszText as LPSTR as LPCSTR,
                -(1 as ::core::ffi::c_int),
                core::ptr::null_mut(),
                0 as ::core::ffi::c_int,
            ) as DWORD;
            newText = heap_alloc(
                (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as *mut WCHAR;
            MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                tvdi.item.pszText as LPSTR as LPCSTR,
                -(1 as ::core::ffi::c_int),
                newText as LPWSTR,
                len as ::core::ffi::c_int,
            );
            iLength = len.wrapping_sub(1 as DWORD) as ::core::ffi::c_int;
        } else {
            newText = tvdi.item.pszText as *mut WCHAR;
        }

        if lstrcmpW(newText as LPCWSTR, (*editedItem).pszText as LPCWSTR)
            != 0 as ::core::ffi::c_int
        {
            let ptr: *mut WCHAR = heap_realloc(
                (*editedItem).pszText as *mut ::core::ffi::c_void,
                (core::mem::size_of::<WCHAR>() as SIZE_T)
                    .wrapping_mul((iLength + 1 as ::core::ffi::c_int) as SIZE_T),
            ) as *mut WCHAR;
            if ptr.is_null() {
                // OutOfMemory, cannot allocate space for label (no trace in C source here)
                if newText != core::ptr::addr_of_mut!(tmpText) as *mut WCHAR {
                    heap_free(newText as *mut ::core::ffi::c_void);
                }
                DestroyWindow((*infoPtr).hwndEdit);
                (*infoPtr).hwndEdit = 0 as HWND;
                (*infoPtr).editItem = 0 as HTREEITEM;
                return FALSE as LRESULT;
            } else {
                (*editedItem).pszText = ptr as LPWSTR;
                (*editedItem).cchTextMax = iLength + 1 as ::core::ffi::c_int;
                lstrcpyW((*editedItem).pszText, newText as LPCWSTR);
                TREEVIEW_ComputeTextWidth(infoPtr, editedItem, 0 as HDC);
            }
        }
        if newText != core::ptr::addr_of_mut!(tmpText) as *mut WCHAR {
            heap_free(newText as *mut ::core::ffi::c_void);
        }
    }

    ShowWindow((*infoPtr).hwndEdit, SW_HIDE as ::core::ffi::c_int);
    DestroyWindow((*infoPtr).hwndEdit);
    (*infoPtr).hwndEdit = 0 as HWND;
    (*infoPtr).editItem = 0 as HTREEITEM;
    return TRUE as LRESULT;
}

unsafe extern "C" fn TREEVIEW_HandleTimer(infoPtr: *mut TREEVIEW_INFO, wParam: WPARAM) -> LRESULT {
    if wParam != TV_EDIT_TIMER as WPARAM {
        return 1 as LRESULT;
    }

    KillTimer((*infoPtr).hwnd, TV_EDIT_TIMER as UINT_PTR);
    (*infoPtr).Timer &= !TV_EDIT_TIMER_SET;

    TREEVIEW_EditLabel(infoPtr, (*infoPtr).selectedItem);

    return 0 as LRESULT;
}

unsafe extern "C" fn TREEVIEW_TrackMouse(
    infoPtr: *const TREEVIEW_INFO,
    mut pt: POINT,
) -> LRESULT {
    let mut cxDrag: INT = GetSystemMetrics((SM_CXDRAG) as i32) as INT;
    let mut cyDrag: INT = GetSystemMetrics((SM_CYDRAG) as i32) as INT;
    let mut r: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut msg: MSG = core::mem::zeroed::<MSG>();

    r.top = (pt.y as INT - cyDrag) as LONG;
    r.left = (pt.x as INT - cxDrag) as LONG;
    r.bottom = (pt.y as INT + cyDrag) as LONG;
    r.right = (pt.x as INT + cxDrag) as LONG;

    SetCapture((*infoPtr).hwnd);

    while 1 != 0 {
        if PeekMessageW(
            core::ptr::addr_of_mut!(msg),
            0 as HWND,
            0 as UINT,
            0 as UINT,
            (PM_REMOVE | PM_NOYIELD) as UINT,
        ) != 0
        {
            if msg.message == WM_MOUSEMOVE as UINT {
                pt.x = (msg.lParam as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                    as ::core::ffi::c_short as LONG;
                pt.y = (msg.lParam as DWORD_PTR >> 16 as ::core::ffi::c_int & 0xffff as DWORD_PTR)
                    as WORD as ::core::ffi::c_short as LONG;
                if PtInRect(core::ptr::addr_of_mut!(r) as LPCRECT, pt) != 0 {
                    continue;
                } else {
                    ReleaseCapture();
                    return 1 as LRESULT;
                }
            } else if msg.message >= WM_LBUTTONDOWN as UINT
                && msg.message <= WM_RBUTTONDBLCLK as UINT
            {
                break;
            }

            DispatchMessageW(core::ptr::addr_of_mut!(msg));
        }

        if GetCapture() != (*infoPtr).hwnd {
            return 0 as LRESULT;
        }
    }

    ReleaseCapture();
    return 0 as LRESULT;
}

unsafe extern "C" fn TREEVIEW_LButtonDoubleClick(infoPtr: *mut TREEVIEW_INFO, lParam: LPARAM) -> LRESULT {
    let mut item: *mut TREEVIEW_ITEM = core::ptr::null_mut();
    let mut hit: TVHITTESTINFO = core::mem::zeroed();

    SetFocus((*infoPtr).hwnd);

    if ((*infoPtr).Timer & (TV_EDIT_TIMER_SET as INT)) != 0 {
        /* If there is pending 'edit label' event - kill it now */
        KillTimer((*infoPtr).hwnd, TV_EDIT_TIMER as UINT_PTR);
    }

    hit.pt.x = (LOWORD!(lParam) as i16) as LONG;
    hit.pt.y = (HIWORD!(lParam) as i16) as LONG;

    item = TREEVIEW_HitTest(infoPtr, core::ptr::addr_of_mut!(hit));
    if item.is_null() {
        return 0 as LRESULT;
    }

    if TREEVIEW_SendSimpleNotify(infoPtr, NM_DBLCLK as UINT) == FALSE as BOOL {
        /* FIXME! */
        let flags: UINT = hit.flags;
        let mut toggle: bool = false;

        if flags == TVHT_ONITEMRIGHT as UINT {
            /* FIXME: we should not have sent NM_DBLCLK in this case. */
        } else if flags == TVHT_ONITEMINDENT as UINT {
            if ((*infoPtr).dwStyle & (TVS_HASLINES as DWORD)) == 0 {
                /* break */
            } else {
                let mut level: INT = (hit.pt.x as UINT).wrapping_div((*infoPtr).uIndent) as INT;
                if ((*infoPtr).dwStyle & (TVS_LINESATROOT as DWORD)) == 0 {
                    level += 1;
                }

                while (*item).iLevel > level {
                    item = (*item).parent as *mut TREEVIEW_ITEM;
                }

                /* fall through */
                toggle = true;
            }
        } else if flags == TVHT_ONITEMLABEL as UINT
            || flags == TVHT_ONITEMICON as UINT
            || flags == TVHT_ONITEMBUTTON as UINT
        {
            toggle = true;
        } else if flags == TVHT_ONITEMSTATEICON as UINT {
            if ((*infoPtr).dwStyle & (TVS_CHECKBOXES as DWORD)) != 0 {
                TREEVIEW_ToggleItemState(infoPtr, item);
            } else {
                TREEVIEW_Toggle(infoPtr, item, TRUE as BOOL);
            }
        }

        if toggle {
            TREEVIEW_Toggle(infoPtr, item, TRUE as BOOL);
        }
    }
    return TRUE as LRESULT;
}

unsafe extern "C" fn TREEVIEW_LButtonDown(
    mut infoPtr: *mut TREEVIEW_INFO,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut do_track: BOOL = 0;
    let mut do_select: BOOL = 0;
    let mut bDoLabelEdit: BOOL = 0;
    let mut hwnd: HWND = (*infoPtr).hwnd;
    let mut ht: TVHITTESTINFO = core::mem::zeroed();

    /* If Edit control is active - kill it and return.
     * The best way to do it is to set focus to itself.
     * Edit control subclassed procedure will automatically call
     * EndEditLabelNow.
     */
    if !(*infoPtr).hwndEdit.is_null() {
        SetFocus(hwnd);
        return 0 as LRESULT;
    }

    ht.pt.x = LOWORD!(lParam as DWORD) as ::core::ffi::c_short as LONG;
    ht.pt.y = HIWORD!(lParam as DWORD) as ::core::ffi::c_short as LONG;

    TREEVIEW_HitTest(infoPtr, core::ptr::addr_of_mut!(ht));

    /* update focusedItem and redraw both items */
    if !ht.hItem.is_null() {
        let mut do_focus: BOOL = 0;

        if TREEVIEW_IsFullRowSelect(infoPtr) != 0 {
            do_focus = (ht.flags
                & (TVHT_ONITEMINDENT | (TVHT_ONITEM as u32) | TVHT_ONITEMRIGHT) as UINT)
                as BOOL;
        } else {
            do_focus = (ht.flags & TVHT_ONITEM as UINT) as BOOL;
        }

        if do_focus != 0 {
            (*infoPtr).focusedItem = (ht.hItem).cast();
            TREEVIEW_InvalidateItem(infoPtr, (*infoPtr).focusedItem.cast());
            TREEVIEW_InvalidateItem(infoPtr, (*infoPtr).selectedItem.cast());
        }
    }

    if (*infoPtr).dwStyle & TVS_DISABLEDRAGDROP as DWORD == 0 {
        if TREEVIEW_IsFullRowSelect(infoPtr) != 0 {
            do_track = (ht.flags
                & (TVHT_ONITEMINDENT | (TVHT_ONITEM as u32) | TVHT_ONITEMRIGHT) as UINT)
                as BOOL;
        } else {
            do_track = (ht.flags & TVHT_ONITEM as UINT) as BOOL;
        }
    } else {
        do_track = FALSE as BOOL;
    }

    /*
     * If the style allows editing and the node is already selected
     * and the click occurred on the item label...
     */
    bDoLabelEdit = (((*infoPtr).dwStyle & TVS_EDITLABELS as DWORD != 0)
        && (ht.flags & TVHT_ONITEMLABEL as UINT != 0)
        && ((*infoPtr).selectedItem == (ht.hItem).cast())) as ::core::ffi::c_int as BOOL;

    'setfocus: {
        /* Send NM_CLICK right away */
        if do_track == 0 && TREEVIEW_SendSimpleNotify(infoPtr, NM_CLICK) != 0 {
            break 'setfocus;
        }

        if ht.flags & TVHT_ONITEMBUTTON as UINT != 0 {
            TREEVIEW_Toggle(infoPtr, (ht.hItem).cast(), TRUE);
            break 'setfocus;
        } else if do_track != 0 {
            /* if TREEVIEW_TrackMouse == 1 dragging occurred and the cursor left the dragged item's rectangle */
            if TREEVIEW_TrackMouse(infoPtr, ht.pt) != 0 {
                TREEVIEW_SendTreeviewDnDNotify(infoPtr, (TVN_BEGINDRAGW) as UINT, (ht.hItem).cast(), ht.pt);
                (*infoPtr).dropItem = (ht.hItem).cast();

                /* clean up focusedItem as we dragged and won't select this item */
                if !(*infoPtr).focusedItem.is_null() {
                    /* refresh the item that was focused */
                    TREEVIEW_InvalidateItem(infoPtr, (*infoPtr).focusedItem.cast());
                    (*infoPtr).focusedItem = core::ptr::null_mut();

                    /* refresh the selected item to return the filled background */
                    TREEVIEW_InvalidateItem(infoPtr, (*infoPtr).selectedItem.cast());
                }

                return 0 as LRESULT;
            }
        }

        if do_track != 0 && TREEVIEW_SendSimpleNotify(infoPtr, NM_CLICK) != 0 {
            break 'setfocus;
        }

        if TREEVIEW_IsFullRowSelect(infoPtr) != 0 {
            do_select = (ht.flags
                & (TVHT_ONITEMINDENT | TVHT_ONITEMICON | TVHT_ONITEMLABEL | TVHT_ONITEMRIGHT) as UINT)
                as BOOL;
        } else {
            do_select = (ht.flags & (TVHT_ONITEMICON | TVHT_ONITEMLABEL) as UINT) as BOOL;
        }

        if bDoLabelEdit != 0 {
            if (*infoPtr).Timer as ::core::ffi::c_int & TV_EDIT_TIMER_SET as ::core::ffi::c_int != 0 {
                KillTimer(hwnd, TV_EDIT_TIMER as UINT_PTR);
            }

            SetTimer(
                hwnd,
                TV_EDIT_TIMER as UINT_PTR,
                GetDoubleClickTime(),
                None,
            );
            (*infoPtr).Timer = (((*infoPtr).Timer as ::core::ffi::c_int)
                | (TV_EDIT_TIMER_SET as ::core::ffi::c_int)) as _;
        } else if do_select != 0 {
            let mut selection: *mut TREEVIEW_ITEM = (*infoPtr).selectedItem as *mut TREEVIEW_ITEM;

            /* Select the current item */
            TREEVIEW_DoSelectItem(infoPtr, (TVGN_CARET) as INT, (ht.hItem).cast(), (TVC_BYMOUSE) as INT);
            TREEVIEW_SingleExpand(infoPtr, (selection).cast(), (ht.hItem).cast());
        } else if ht.flags & TVHT_ONITEMSTATEICON as UINT != 0 {
            /* TVS_CHECKBOXES requires us to toggle the current state */
            if (*infoPtr).dwStyle & TVS_CHECKBOXES as DWORD != 0 {
                TREEVIEW_ToggleItemState(infoPtr, (ht.hItem).cast());
            }
        }
    }

    SetFocus(hwnd);
    return 0 as LRESULT;
}

unsafe extern "C" fn TREEVIEW_RButtonDown(
    mut infoPtr: *mut TREEVIEW_INFO,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut ht: TVHITTESTINFO = core::mem::zeroed();

    if !(*infoPtr).hwndEdit.is_null() {
        SetFocus((*infoPtr).hwnd);
        return 0;
    }

    ht.pt.x = LOWORD!(lParam as DWORD_PTR) as ::core::ffi::c_short as LONG;
    ht.pt.y = HIWORD!(lParam as DWORD_PTR) as ::core::ffi::c_short as LONG;

    if !TREEVIEW_HitTest(infoPtr, core::ptr::addr_of_mut!(ht)).is_null() {
        (*infoPtr).focusedItem = (ht.hItem).cast();
        TREEVIEW_InvalidateItem(infoPtr, (*infoPtr).focusedItem as *const TREEVIEW_ITEM);
        TREEVIEW_InvalidateItem(infoPtr, (*infoPtr).selectedItem as *const TREEVIEW_ITEM);
    }

    if TREEVIEW_TrackMouse(infoPtr, ht.pt) != 0 {
        if !ht.hItem.is_null() {
            TREEVIEW_SendTreeviewDnDNotify(infoPtr, TVN_BEGINRDRAGW as UINT, (ht.hItem).cast(), ht.pt);
            (*infoPtr).dropItem = (ht.hItem).cast();
        }
    } else {
        SetFocus((*infoPtr).hwnd);
        if TREEVIEW_SendSimpleNotify(infoPtr, NM_RCLICK as UINT) == 0 {
            /* Send a WM_CONTEXTMENU message in response to the RBUTTONUP */
            SendMessageW(
                (*infoPtr).hwndNotify,
                WM_CONTEXTMENU as UINT,
                (*infoPtr).hwnd as WPARAM,
                GetMessagePos() as LPARAM,
            );
        }
    }

    if !ht.hItem.is_null() {
        TREEVIEW_InvalidateItem(infoPtr, (*infoPtr).focusedItem as *const TREEVIEW_ITEM);
        (*infoPtr).focusedItem = (*infoPtr).selectedItem;
        TREEVIEW_InvalidateItem(infoPtr, (*infoPtr).focusedItem as *const TREEVIEW_ITEM);
    }

    return 0;
}

unsafe extern "C" fn TREEVIEW_CreateDragImage(
    mut infoPtr: *mut TREEVIEW_INFO,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut dragItem: *mut TREEVIEW_ITEM = lParam as *mut TREEVIEW_ITEM;
    let mut cx: INT = 0;
    let mut cy: INT = 0;
    let mut hdc: HDC = core::ptr::null_mut();
    let mut htopdc: HDC = core::ptr::null_mut();
    let mut hwtop: HWND = core::ptr::null_mut();
    let mut hbmp: HBITMAP = core::ptr::null_mut();
    let mut hOldbmp: HBITMAP = core::ptr::null_mut();
    let mut size: SIZE = SIZE { cx: 0, cy: 0 };
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut hOldFont: HFONT = core::ptr::null_mut();

    if (*infoPtr).himlNormal.is_null() {
        return 0;
    }

    if dragItem.is_null() || TREEVIEW_ValidItem(infoPtr, dragItem as HTREEITEM) == 0 {
        return 0;
    }

    TREEVIEW_UpdateDispInfo(infoPtr, dragItem, TVIF_TEXT);

    hwtop = GetDesktopWindow();
    htopdc = GetDC(hwtop);
    hdc = CreateCompatibleDC(htopdc);

    hOldFont = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;

    if !(*dragItem).pszText.is_null() {
        GetTextExtentPoint32W(
            hdc,
            (*dragItem).pszText as LPCWSTR,
            lstrlenW((*dragItem).pszText as LPCWSTR),
            core::ptr::addr_of_mut!(size),
        );
    } else {
        GetTextExtentPoint32A(
            hdc,
            b"\0".as_ptr() as LPCSTR,
            0,
            core::ptr::addr_of_mut!(size),
        );
    }

    hbmp = CreateCompatibleBitmap(htopdc, size.cx, size.cy);
    hOldbmp = SelectObject(hdc, hbmp as HGDIOBJ) as HBITMAP;

    ImageList_GetIconSize(
        (*infoPtr).himlNormal,
        core::ptr::addr_of_mut!(cx),
        core::ptr::addr_of_mut!(cy),
    );
    size.cx += cx;
    if cy > size.cy {
        size.cy = cy;
    }

    (*infoPtr).dragList = ImageList_Create(
        size.cx,
        size.cy,
        ILC_COLOR,
        10,
        10,
    );
    ImageList_Draw(
        (*infoPtr).himlNormal,
        (*dragItem).iImage,
        hdc,
        0,
        0,
        ILD_NORMAL,
    );

    /* draw item text */

    SetRect(
        core::ptr::addr_of_mut!(rc),
        cx,
        0,
        size.cx,
        size.cy,
    );

    if !(*dragItem).pszText.is_null() {
        DrawTextW(
            hdc,
            (*dragItem).pszText as LPCWSTR,
            lstrlenW((*dragItem).pszText as LPCWSTR),
            core::ptr::addr_of_mut!(rc),
            DT_LEFT,
        );
    }

    SelectObject(hdc, hOldFont as HGDIOBJ);
    SelectObject(hdc, hOldbmp as HGDIOBJ);

    ImageList_Add((*infoPtr).dragList, hbmp, core::ptr::null_mut());

    DeleteDC(hdc);
    DeleteObject(hbmp as HGDIOBJ);
    ReleaseDC(hwtop, htopdc);

    return (*infoPtr).dragList as LRESULT;
}

unsafe extern "C" fn TREEVIEW_DoSelectItem(
    mut infoPtr: *mut TREEVIEW_INFO,
    mut action: INT,
    mut newSelect: HTREEITEM,
    mut cause: INT,
) -> LRESULT {
    let mut prevSelect: *mut TREEVIEW_ITEM = core::ptr::null_mut::<TREEVIEW_ITEM>();

    assert!((newSelect.is_null() || TREEVIEW_ValidItem(infoPtr, newSelect) != 0) as i32);

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TREEVIEW_DoSelectItem\0" as *const u8 as *const ::core::ffi::c_char,
            4489 as ::core::ffi::c_int,
            b"Entering item %p (%s), flag 0x%x, cause 0x%x, state 0x%x\n\0" as *const u8
                as *const ::core::ffi::c_char,
            newSelect,
            TREEVIEW_ItemName(newSelect as *const TREEVIEW_ITEM),
            action,
            cause,
            if !newSelect.is_null() { (*newSelect).state } else { 0 as UINT },
        );
    }

    /* reset and redraw focusedItem if focusedItem was set so we don't */
    /* have to worry about the previously focused item when we set a new one */
    TREEVIEW_InvalidateItem(infoPtr, (*infoPtr).focusedItem as *const TREEVIEW_ITEM);
    (*infoPtr).focusedItem = core::ptr::null_mut::<_TREEITEM>();

    let mut current_block_38: u64;
    match (action) as u32 {
        // C `case TVGN_CARET|TVSI_NOSINGLEEXPAND:` is ONE constant (0x8009); as a Rust pattern
        // `A | B` is an or-pattern that matched TVGN_CARET and 0x8000 instead.
        x if x == TVGN_CARET | TVSI_NOSINGLEEXPAND => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/treeview.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TREEVIEW_DoSelectItem\0" as *const u8
                        as *const ::core::ffi::c_char,
                    4499 as ::core::ffi::c_int,
                    b"TVSI_NOSINGLEEXPAND specified.\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            /* Fall through */
            current_block_38 = 3800187404441252834;
        }
        TVGN_CARET => {
            current_block_38 = 3800187404441252834;
        }
        TVGN_DROPHILITE => {
            prevSelect = (*infoPtr).dropItem as *mut TREEVIEW_ITEM;
            if !prevSelect.is_null() {
                (*prevSelect).state &= !(TVIS_DROPHILITED as UINT);
            }
            (*infoPtr).dropItem = newSelect;
            if !newSelect.is_null() {
                (*newSelect).state |= TVIS_DROPHILITED as UINT;
            }
            TREEVIEW_Invalidate(infoPtr, prevSelect);
            TREEVIEW_Invalidate(infoPtr, newSelect as *const TREEVIEW_ITEM);
            current_block_38 = 7245201122033322888;
        }
        TVGN_FIRSTVISIBLE => {
            if !newSelect.is_null() {
                TREEVIEW_EnsureVisible(infoPtr, newSelect, FALSE);
                TREEVIEW_SetFirstVisible(infoPtr, newSelect as *mut TREEVIEW_ITEM, TRUE);
                TREEVIEW_Invalidate(infoPtr, core::ptr::null::<TREEVIEW_ITEM>());
            }
            current_block_38 = 7245201122033322888;
        }
        _ => {
            current_block_38 = 7245201122033322888;
        }
    }

    match current_block_38 {
        3800187404441252834 => {
            prevSelect = (*infoPtr).selectedItem as *mut TREEVIEW_ITEM;
            if prevSelect == (newSelect).cast() {
                TREEVIEW_EnsureVisible(infoPtr, (*infoPtr).selectedItem, FALSE);
            } else {
                if TREEVIEW_SendTreeviewNotify(
                    infoPtr,
                    (TVN_SELCHANGINGW) as UINT,
                    cause as UINT,
                    TVIF_TEXT | TVIF_HANDLE | TVIF_STATE | TVIF_PARAM,
                    prevSelect as HTREEITEM,
                    newSelect,
                ) != 0
                {
                    return FALSE as LRESULT;
                }
                if !prevSelect.is_null() {
                    (*prevSelect).state &= !(TVIS_SELECTED as UINT);
                }
                if !newSelect.is_null() {
                    (*newSelect).state |= TVIS_SELECTED as UINT;
                }
                (*infoPtr).selectedItem = newSelect;
                TREEVIEW_EnsureVisible(infoPtr, (*infoPtr).selectedItem, FALSE);
                TREEVIEW_InvalidateItem(infoPtr, prevSelect as *const TREEVIEW_ITEM);
                TREEVIEW_InvalidateItem(infoPtr, newSelect as *const TREEVIEW_ITEM);
                TREEVIEW_SendTreeviewNotify(
                    infoPtr,
                    (TVN_SELCHANGEDW) as UINT,
                    cause as UINT,
                    TVIF_TEXT | TVIF_HANDLE | TVIF_STATE | TVIF_PARAM,
                    prevSelect as HTREEITEM,
                    newSelect,
                );
            }
        }
        _ => {}
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
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TREEVIEW_DoSelectItem\0" as *const u8 as *const ::core::ffi::c_char,
            4562 as ::core::ffi::c_int,
            b"Leaving state 0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
            if !newSelect.is_null() { (*newSelect).state } else { 0 as UINT },
        );
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn TREEVIEW_SelectItem(
    mut infoPtr: *mut TREEVIEW_INFO,
    mut wParam: INT,
    mut item: HTREEITEM,
) -> LRESULT {
    let mut selection: *mut TREEVIEW_ITEM = (*infoPtr).selectedItem as *mut TREEVIEW_ITEM;

    if !item.is_null() && TREEVIEW_ValidItem(infoPtr, item) == 0 {
        return (FALSE) as LRESULT;
    }

    if item == (*infoPtr).selectedItem {
        return (TRUE) as LRESULT;
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
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TREEVIEW_SelectItem\0" as *const u8 as *const ::core::ffi::c_char,
            4578 as ::core::ffi::c_int,
            b"%p (%s) %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            item,
            TREEVIEW_ItemName(item as *const TREEVIEW_ITEM),
            wParam,
        );
    }

    if TREEVIEW_DoSelectItem(infoPtr, wParam, item, (TVC_UNKNOWN) as INT) == 0 {
        return (FALSE) as LRESULT;
    }

    TREEVIEW_SingleExpand(infoPtr, selection as HTREEITEM, item);

    return (TRUE) as LRESULT;
}

unsafe extern "C" fn TREEVIEW_ProcessLetterKeys(mut infoPtr: *mut TREEVIEW_INFO, mut charCode: WPARAM, mut keyData: LPARAM) -> INT {
    let mut nItem: HTREEITEM = core::ptr::null_mut();
    let mut endidx: HTREEITEM;
    let mut idx: HTREEITEM;
    let mut item: TVITEMEXW = core::mem::zeroed();
    let mut buffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut timestamp: DWORD;
    let mut elapsed: DWORD;

    /* simple parameter checking */
    if charCode == 0 || keyData == 0 {
        return 0 as INT;
    }

    /* only allow the valid WM_CHARs through */
    if isalnum(charCode as ::core::ffi::c_int) == 0
        && charCode != '.' as WPARAM
        && charCode != '`' as WPARAM
        && charCode != '!' as WPARAM
        && charCode != '@' as WPARAM
        && charCode != '#' as WPARAM
        && charCode != '$' as WPARAM
        && charCode != '%' as WPARAM
        && charCode != '^' as WPARAM
        && charCode != '&' as WPARAM
        && charCode != '*' as WPARAM
        && charCode != '(' as WPARAM
        && charCode != ')' as WPARAM
        && charCode != '-' as WPARAM
        && charCode != '_' as WPARAM
        && charCode != '+' as WPARAM
        && charCode != '=' as WPARAM
        && charCode != '\\' as WPARAM
        && charCode != ']' as WPARAM
        && charCode != '}' as WPARAM
        && charCode != '[' as WPARAM
        && charCode != '{' as WPARAM
        && charCode != '/' as WPARAM
        && charCode != '?' as WPARAM
        && charCode != '>' as WPARAM
        && charCode != '<' as WPARAM
        && charCode != ',' as WPARAM
        && charCode != '~' as WPARAM
    {
        return 0 as INT;
    }

    /* compute how much time elapsed since last keypress */
    timestamp = GetTickCount();
    if timestamp > (*infoPtr).lastKeyPressTimestamp {
        elapsed = timestamp.wrapping_sub((*infoPtr).lastKeyPressTimestamp);
    } else {
        elapsed = (*infoPtr).lastKeyPressTimestamp.wrapping_sub(timestamp);
    }

    /* update the search parameters */
    (*infoPtr).lastKeyPressTimestamp = timestamp;
    if elapsed < KEY_DELAY as DWORD {
        if ((*infoPtr).nSearchParamLength as usize)
            < ARRAY_SIZE!((*infoPtr).szSearchParam) as usize
        {
            let fresh = (*infoPtr).nSearchParamLength;
            (*infoPtr).nSearchParamLength = (*infoPtr).nSearchParamLength + 1;
            (*infoPtr).szSearchParam[fresh as usize] = charCode as WCHAR;
        }
        if (*infoPtr).charCode != charCode {
            charCode = 0 as WPARAM;
            (*infoPtr).charCode = charCode;
        }
    } else {
        (*infoPtr).charCode = charCode;
        (*infoPtr).szSearchParam[0 as usize] = charCode as WCHAR;
        (*infoPtr).nSearchParamLength = 1 as INT;
        /* Redundant with the 1 char string */
        charCode = 0 as WPARAM;
    }

    /* and search from the current position */
    nItem = core::ptr::null_mut();
    if !(*infoPtr).selectedItem.is_null() {
        endidx = (*infoPtr).selectedItem;
        /* if looking for single character match,
         * then we must always move forward
         */
        if (*infoPtr).nSearchParamLength == 1 as INT {
            idx = TREEVIEW_GetNextListItem(infoPtr, endidx as *const TREEVIEW_ITEM) as HTREEITEM;
        } else {
            idx = endidx;
        }
    } else {
        endidx = core::ptr::null_mut();
        idx = (*(*infoPtr).root).firstChild;
    }
    loop {
        /* At the end point, sort out wrapping */
        if idx.is_null() {

            /* If endidx is null, stop at the last item (ie top to bottom) */
            if endidx.is_null() {
                break;
            }

            /* Otherwise, start again at the very beginning */
            idx = (*(*infoPtr).root).firstChild;

            /* But if we are stopping on the first child, end now! */
            if idx == endidx {
                break;
            }
        }

        /* get item */
        ZeroMemory!(
            &raw mut item as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<TVITEMEXW>() as usize,
        );
        item.mask = TVIF_TEXT as UINT;
        item.hItem = (idx).cast();
        item.pszText = &raw mut buffer as *mut WCHAR as LPWSTR;
        item.cchTextMax = ARRAY_SIZE!(buffer) as ::core::ffi::c_int;
        TREEVIEW_GetItemT(infoPtr, &raw mut item, TRUE as BOOL);

        /* check for a match */
        if wcsnicmp(
            item.pszText as *const WCHAR,
            &raw mut (*infoPtr).szSearchParam as *mut WCHAR,
            (*infoPtr).nSearchParamLength as usize,
        ) == 0 as ::core::ffi::c_int
        {
            nItem = idx;
            break;
        } else if (charCode != 0 as WPARAM)
            && nItem.is_null()
            && (nItem != (*infoPtr).selectedItem)
            && (wcsnicmp(
                item.pszText as *const WCHAR,
                &raw mut (*infoPtr).szSearchParam as *mut WCHAR,
                1 as usize,
            ) == 0 as ::core::ffi::c_int)
        {
            /* This would work but we must keep looking for a longer match */
            nItem = idx;
        }
        idx = TREEVIEW_GetNextListItem(infoPtr, idx as *const TREEVIEW_ITEM) as HTREEITEM;
        if !(idx != endidx) {
            break;
        }
    }

    if !nItem.is_null() {
        if TREEVIEW_DoSelectItem(infoPtr, TVGN_CARET as INT, nItem, TVC_BYKEYBOARD as INT) != 0 {
            TREEVIEW_EnsureVisible(infoPtr, nItem, FALSE as BOOL);
        }
    }

    return 0 as INT;
}

unsafe extern "C" fn TREEVIEW_EnsureVisible(infoPtr: *mut TREEVIEW_INFO, item: HTREEITEM, bHScroll: BOOL) -> LRESULT {
    let mut viscount: i32 = 0;
    let hasFirstVisible: BOOL = ((*infoPtr).firstVisible != core::ptr::null_mut()) as i32;
    let mut newFirstVisible: HTREEITEM = core::ptr::null_mut();
    let mut visible_pos: i32 = -(1);

    if TREEVIEW_ValidItem(infoPtr, item) == 0 {
        return (FALSE) as LRESULT;
    }

    if ISVISIBLE!(item) == false {
        /* Expand parents as necessary. */
        let mut parent: HTREEITEM;

        /* see if we are trying to ensure that root is visible */
        if (item != (*infoPtr).root) && TREEVIEW_ValidItem(infoPtr, item) != 0 {
            parent = (*item).parent;
        } else {
            parent = item; /* this item is the topmost item */
        }

        while parent != (*infoPtr).root {
            if ((*parent).state & (TVIS_EXPANDED as u32)) == 0 {
                TREEVIEW_Expand(infoPtr, (parent).cast(), FALSE, TRUE);
            }

            parent = (*parent).parent;
        }
    }

    viscount = TREEVIEW_GetVisibleCount(infoPtr) as i32;

    if hasFirstVisible != 0 {
        visible_pos = ((*item).visibleOrder - (*(*infoPtr).firstVisible).visibleOrder) as i32;
    }

    if visible_pos < 0 {
        /* item is before the start of the list: put it at the top. */
        newFirstVisible = item;
    } else if visible_pos >= viscount
        /* Sometimes, before we are displayed, GVC is 0, causing us to
         * spuriously scroll up. */
        && visible_pos > 0
        && ((*infoPtr).dwStyle & (TVS_NOSCROLL as DWORD)) == 0
    {
        /* item is past the end of the list. */
        let scroll: i32 = visible_pos - viscount;

        newFirstVisible = (TREEVIEW_GetListItem(
            infoPtr,
            ((*infoPtr).firstVisible).cast(),
            (scroll + 1) as LONG,
        )).cast();
    }

    if bHScroll != 0 {
        /* Scroll window so item's text is visible as much as possible */
        /* Calculation of amount of extra space is taken from EditLabel code */
        let mut pos: i32;
        let mut x: i32;
        let mut textMetric: TEXTMETRICW = core::mem::zeroed();
        let hdc: HDC = GetWindowDC((*infoPtr).hwnd);

        x = (*item).textWidth as i32;

        GetTextMetricsW(hdc, core::ptr::addr_of_mut!(textMetric));
        ReleaseDC((*infoPtr).hwnd, hdc);

        x += (textMetric.tmMaxCharWidth as i32) * 2;
        x = max(x, (textMetric.tmMaxCharWidth as i32) * 3);

        if (*item).textOffset < 0 {
            pos = (*item).textOffset as i32;
        } else if (*item).textOffset + (x as LONG) > (*infoPtr).clientWidth {
            if (x as LONG) > (*infoPtr).clientWidth {
                pos = (*item).textOffset as i32;
            } else {
                pos = ((*item).textOffset + (x as LONG) - (*infoPtr).clientWidth) as i32;
            }
        } else {
            pos = 0;
        }

        TREEVIEW_HScroll(
            infoPtr,
            MAKEWPARAM!(
                SB_THUMBPOSITION as WORD,
                ((*infoPtr).scrollX + (pos as LONG)) as WORD,
            ),
        );
    }

    if !newFirstVisible.is_null() && newFirstVisible != (*infoPtr).firstVisible {
        TREEVIEW_SetFirstVisible(infoPtr, (newFirstVisible).cast(), TRUE);

        return (TRUE) as LRESULT;
    }

    return (FALSE) as LRESULT;
}

unsafe extern "C" fn TREEVIEW_SetFirstVisible(
    mut infoPtr: *mut TREEVIEW_INFO,
    mut newFirstVisible: *mut TREEVIEW_ITEM,
    mut bUpdateScrollPos: BOOL,
) {
    let mut gap_size: ::core::ffi::c_int = 0;
    if !newFirstVisible.is_null() {
        gap_size = (TREEVIEW_GetVisibleCount(infoPtr) - (*infoPtr).maxVisibleOrder as LRESULT
            + (*newFirstVisible).visibleOrder as LRESULT) as ::core::ffi::c_int;
        if gap_size > 0 as ::core::ffi::c_int {
            newFirstVisible = TREEVIEW_GetListItem(infoPtr, newFirstVisible, -(gap_size as LONG));
            if newFirstVisible.is_null() {
                newFirstVisible = (*(*infoPtr).root).firstChild as *mut TREEVIEW_ITEM;
            }
        }
    }
    if (*infoPtr).firstVisible != newFirstVisible as HTREEITEM {
        if (*infoPtr).firstVisible.is_null() || newFirstVisible.is_null() {
            (*infoPtr).firstVisible = newFirstVisible as HTREEITEM;
            TREEVIEW_Invalidate(infoPtr, ::core::ptr::null::<TREEVIEW_ITEM>());
        } else {
            let mut item: *mut TREEVIEW_ITEM = ::core::ptr::null_mut::<TREEVIEW_ITEM>();
            let mut scroll: ::core::ffi::c_int = (*infoPtr).uItemHeight.wrapping_mul(
                ((*(*infoPtr).firstVisible).visibleOrder - (*newFirstVisible).visibleOrder) as UINT,
            ) as ::core::ffi::c_int;
            (*infoPtr).firstVisible = newFirstVisible as HTREEITEM;
            item = (*(*infoPtr).root).firstChild as *mut TREEVIEW_ITEM;
            while !item.is_null() {
                (*item).rect.top += scroll;
                (*item).rect.bottom += scroll;
                item = TREEVIEW_GetNextListItem(infoPtr, item);
            }
            if bUpdateScrollPos != 0 {
                SetScrollPos(
                    (*infoPtr).hwnd,
                    (SB_VERT) as i32,
                    (*newFirstVisible).visibleOrder as ::core::ffi::c_int,
                    TRUE,
                );
            }
            ScrollWindowEx(
                (*infoPtr).hwnd,
                0 as ::core::ffi::c_int,
                scroll,
                ::core::ptr::null::<RECT>(),
                ::core::ptr::null::<RECT>(),
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                (SW_ERASE | SW_INVALIDATE) as UINT,
            );
        }
    }
}

unsafe extern "C" fn TREEVIEW_VScroll(infoPtr: *mut TREEVIEW_INFO, wParam: WPARAM) -> LRESULT {
    let mut oldFirstVisible: *mut TREEVIEW_ITEM = (*infoPtr).firstVisible as *mut TREEVIEW_ITEM;
    let mut newFirstVisible: *mut TREEVIEW_ITEM = core::ptr::null_mut::<TREEVIEW_ITEM>();

    let mut nScrollCode: ::core::ffi::c_int = LOWORD!(wParam) as ::core::ffi::c_int;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TREEVIEW_VScroll\0" as *const u8 as *const ::core::ffi::c_char,
            4906 as ::core::ffi::c_int,
            b"wp %lx\n\0" as *const u8 as *const ::core::ffi::c_char,
            wParam,
        );
    }

    if !((*infoPtr).uInternalStatus & (TV_VSCROLL as u32) != 0) {
        return 0;
    }

    if oldFirstVisible.is_null() {
        assert!(((*(*infoPtr).root).firstChild.is_null()) as i32);
        return 0;
    }

    match (nScrollCode) as u32 {
        SB_TOP => {
            newFirstVisible = (*(*infoPtr).root).firstChild as *mut TREEVIEW_ITEM;
        }
        SB_BOTTOM => {
            newFirstVisible = TREEVIEW_GetLastListItem(infoPtr, (*infoPtr).root as *mut TREEVIEW_ITEM);
        }
        SB_LINEUP => {
            newFirstVisible = TREEVIEW_GetPrevListItem(infoPtr, oldFirstVisible);
        }
        SB_LINEDOWN => {
            newFirstVisible = TREEVIEW_GetNextListItem(infoPtr, oldFirstVisible);
        }
        SB_PAGEUP => {
            newFirstVisible = TREEVIEW_GetListItem(
                infoPtr,
                oldFirstVisible,
                -max(1, TREEVIEW_GetVisibleCount(infoPtr)) as LONG,
            );
        }
        SB_PAGEDOWN => {
            newFirstVisible = TREEVIEW_GetListItem(
                infoPtr,
                oldFirstVisible,
                max(1, TREEVIEW_GetVisibleCount(infoPtr)) as LONG,
            );
        }
        SB_THUMBTRACK | SB_THUMBPOSITION => {
            newFirstVisible = TREEVIEW_GetListItem(
                infoPtr,
                (*(*infoPtr).root).firstChild as *mut TREEVIEW_ITEM,
                HIWORD!(wParam) as WORD as SHORT as LONG,
            );
        }
        SB_ENDSCROLL => {
            return 0;
        }
        _ => {}
    }

    if !newFirstVisible.is_null() {
        if newFirstVisible != oldFirstVisible {
            TREEVIEW_SetFirstVisible(
                infoPtr,
                newFirstVisible,
                ((nScrollCode) as u32 != SB_THUMBTRACK) as ::core::ffi::c_int,
            );
        } else if (nScrollCode) as u32 == SB_THUMBPOSITION {
            SetScrollPos(
                (*infoPtr).hwnd,
                (SB_VERT) as i32,
                (*newFirstVisible).visibleOrder as ::core::ffi::c_int,
                TRUE,
            );
        }
    }

    return 0;
}

unsafe extern "C" fn TREEVIEW_HScroll(mut infoPtr: *mut TREEVIEW_INFO, mut wParam: WPARAM) -> LRESULT {
    let mut maxWidth: i32;
    let mut scrollX: i32 = (*infoPtr).scrollX as i32;
    let mut nScrollCode: i32 = LOWORD!(wParam) as i32;

    if (*infoPtr).uInternalStatus & (TV_HSCROLL as UINT) == 0 {
        return FALSE as LRESULT;
    }

    maxWidth = (*infoPtr).treeWidth - (*infoPtr).clientWidth;
    'scroll: {
        /* shall never occur */
        if maxWidth <= 0 as i32 {
            scrollX = 0 as i32;
            break 'scroll;
        }

        match (nScrollCode) as u32 {
            SB_LINELEFT => {
                scrollX = (scrollX as UINT).wrapping_sub((*infoPtr).uItemHeight) as i32;
            }
            SB_LINERIGHT => {
                scrollX = (scrollX as UINT).wrapping_add((*infoPtr).uItemHeight) as i32;
            }
            SB_PAGELEFT => {
                scrollX -= (*infoPtr).clientWidth as i32;
            }
            SB_PAGERIGHT => {
                scrollX += (*infoPtr).clientWidth as i32;
            }

            SB_THUMBTRACK | SB_THUMBPOSITION => {
                scrollX = HIWORD!(wParam) as SHORT as i32;
            }

            SB_ENDSCROLL => {
                return 0 as LRESULT;
            }
            _ => {}
        }

        if scrollX > maxWidth {
            scrollX = maxWidth;
        } else if scrollX < 0 as i32 {
            scrollX = 0 as i32;
        }
    }

    if scrollX != (*infoPtr).scrollX as i32 {
        let mut item: *mut TREEVIEW_ITEM = core::ptr::null_mut::<TREEVIEW_ITEM>();
        let mut scroll_pixels: LONG = (*infoPtr).scrollX - scrollX;

        item = (*(*infoPtr).root).firstChild as *mut TREEVIEW_ITEM;
        while !item.is_null() {
            (*item).linesOffset += scroll_pixels;
            (*item).stateOffset += scroll_pixels;
            (*item).imageOffset += scroll_pixels;
            (*item).textOffset += scroll_pixels;
            item = TREEVIEW_GetNextListItem(infoPtr, item);
        }

        ScrollWindow(
            (*infoPtr).hwnd,
            scroll_pixels,
            0,
            core::ptr::null::<RECT>(),
            core::ptr::null::<RECT>(),
        );
        (*infoPtr).scrollX = scrollX as LONG;
        UpdateWindow((*infoPtr).hwnd);
    }

    if (nScrollCode) as u32 != SB_THUMBTRACK {
        SetScrollPos(
            (*infoPtr).hwnd,
            (SB_HORZ) as i32,
            scrollX,
            TRUE,
        );
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TREEVIEW_MouseWheel(infoPtr: *mut TREEVIEW_INFO, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut wheelDelta: i16 = 0;
    let mut pulScrollLines: INT = 3;

    if wParam & ((MK_SHIFT | MK_CONTROL) as WPARAM) != 0 {
        return DefWindowProcW((*infoPtr).hwnd, WM_MOUSEWHEEL as UINT, wParam, lParam);
    }

    if (*infoPtr).firstVisible.is_null() {
        return TRUE as LRESULT;
    }

    SystemParametersInfoW(
        SPI_GETWHEELSCROLLLINES as UINT,
        0 as UINT,
        core::ptr::addr_of_mut!(pulScrollLines) as PVOID,
        0 as UINT,
    );

    wheelDelta = GET_WHEEL_DELTA_WPARAM!(wParam);
    /* if scrolling changes direction, ignore left overs */
    if ((wheelDelta as INT) < 0 && (*infoPtr).wheelRemainder < 0)
        || ((wheelDelta as INT) > 0 && (*infoPtr).wheelRemainder > 0)
    {
        (*infoPtr).wheelRemainder += wheelDelta as INT;
    } else {
        (*infoPtr).wheelRemainder = wheelDelta as INT;
    }

    if (*infoPtr).wheelRemainder != 0 && pulScrollLines != 0 {
        let mut newDy: INT = 0;
        let mut maxDy: INT = 0;
        let mut lineScroll: INT = 0;

        lineScroll = pulScrollLines * (*infoPtr).wheelRemainder / (WHEEL_DELTA) as i32;
        (*infoPtr).wheelRemainder -= 120 /* WHEEL_DELTA */ * lineScroll / pulScrollLines;

        newDy = (*(*infoPtr).firstVisible).visibleOrder - lineScroll;
        maxDy = (*infoPtr).maxVisibleOrder;

        if newDy > maxDy {
            newDy = maxDy;
        }

        if newDy < 0 {
            newDy = 0;
        }

        TREEVIEW_VScroll(infoPtr, MAKEWPARAM!(SB_THUMBPOSITION as WORD, newDy as WORD));
    }
    return TRUE as LRESULT;
}

unsafe extern "C" fn TREEVIEW_Create(hwnd: HWND, lpcs: *const CREATESTRUCTW) -> LRESULT {
    let mut rcClient: RECT = core::mem::zeroed();
    let mut infoPtr: *mut TREEVIEW_INFO = core::ptr::null_mut::<TREEVIEW_INFO>();
    let mut lf: LOGFONTW = core::mem::zeroed();

    infoPtr = heap_alloc_zero(core::mem::size_of::<TREEVIEW_INFO>() as SIZE_T) as *mut TREEVIEW_INFO;

    if infoPtr.is_null() {
        return 0 as LRESULT;
    }

    SetWindowLongPtrW(hwnd, 0, infoPtr as DWORD_PTR as LONG_PTR);

    (*infoPtr).hwnd = hwnd;
    (*infoPtr).dwStyle = GetWindowLongW(hwnd, GWL_STYLE) as DWORD;
    (*infoPtr).Timer = 0 as INT;
    (*infoPtr).uNumItems = 0 as UINT;
    (*infoPtr).cdmode = 0 as INT;
    (*infoPtr).uScrollTime = 300 as UINT; /* milliseconds */
    (*infoPtr).bRedraw = TRUE as BOOL;

    GetClientRect(hwnd, core::ptr::addr_of_mut!(rcClient));

    /* No scroll bars yet. */
    (*infoPtr).clientWidth = rcClient.right;
    (*infoPtr).clientHeight = rcClient.bottom;
    (*infoPtr).uInternalStatus = 0 as UINT;

    (*infoPtr).treeWidth = 0 as LONG;
    (*infoPtr).treeHeight = 0 as LONG;

    (*infoPtr).uIndent = MINIMUM_INDENT as UINT;
    (*infoPtr).selectedItem = core::ptr::null_mut::<_TREEITEM>();
    (*infoPtr).focusedItem = core::ptr::null_mut::<_TREEITEM>();
    (*infoPtr).hotItem = core::ptr::null_mut::<_TREEITEM>();
    (*infoPtr).editItem = core::ptr::null_mut::<_TREEITEM>();
    (*infoPtr).firstVisible = core::ptr::null_mut::<_TREEITEM>();
    (*infoPtr).maxVisibleOrder = 0 as LONG;
    (*infoPtr).dropItem = core::ptr::null_mut::<_TREEITEM>();
    (*infoPtr).insertMarkItem = core::ptr::null_mut::<_TREEITEM>();
    (*infoPtr).insertBeforeorAfter = 0 as BOOL;
    /* dragList */

    (*infoPtr).scrollX = 0 as LONG;
    (*infoPtr).wheelRemainder = 0 as INT;

    (*infoPtr).clrBk = CLR_NONE as COLORREF; /* use system color */
    (*infoPtr).clrText = CLR_NONE as COLORREF; /* use system color */
    (*infoPtr).clrLine = CLR_DEFAULT as COLORREF;
    (*infoPtr).clrInsertMark = CLR_DEFAULT as COLORREF;

    /* hwndToolTip */

    (*infoPtr).hwndEdit = core::ptr::null_mut::<HWND__>();
    (*infoPtr).wpEditOrig = None;
    (*infoPtr).bIgnoreEditKillFocus = FALSE as BOOL;
    (*infoPtr).bLabelChanged = FALSE as BOOL;

    (*infoPtr).himlNormal = core::ptr::null_mut::<_IMAGELIST>();
    (*infoPtr).himlState = core::ptr::null_mut::<_IMAGELIST>();
    (*infoPtr).normalImageWidth = 0 as core::ffi::c_int;
    (*infoPtr).normalImageHeight = 0 as core::ffi::c_int;
    (*infoPtr).stateImageWidth = 0 as core::ffi::c_int;
    (*infoPtr).stateImageHeight = 0 as core::ffi::c_int;

    (*infoPtr).items = DPA_Create(16 as core::ffi::c_int);

    SystemParametersInfoW(
        SPI_GETICONTITLELOGFONT as UINT,
        core::mem::size_of::<LOGFONTW>() as UINT,
        core::ptr::addr_of_mut!(lf) as PVOID,
        0 as UINT,
    );
    (*infoPtr).hDefaultFont = CreateFontIndirectW(core::ptr::addr_of_mut!(lf));
    (*infoPtr).hFont = (*infoPtr).hDefaultFont;
    (*infoPtr).hBoldFont = TREEVIEW_CreateBoldFont((*infoPtr).hFont);
    (*infoPtr).hUnderlineFont = TREEVIEW_CreateUnderlineFont((*infoPtr).hFont);
    (*infoPtr).hBoldUnderlineFont = TREEVIEW_CreateBoldUnderlineFont((*infoPtr).hFont);
    (*infoPtr).hcurHand = LoadCursorW(
        core::ptr::null_mut::<HINSTANCE__>(),
        IDC_HAND as LPWSTR as LPCWSTR,
    );

    (*infoPtr).uItemHeight = TREEVIEW_NaturalHeight(infoPtr);

    (*infoPtr).root = TREEVIEW_AllocateItem(infoPtr) as HTREEITEM;
    (*(*infoPtr).root).state = TVIS_EXPANDED as UINT;
    (*(*infoPtr).root).iLevel = -(1 as core::ffi::c_int);
    (*(*infoPtr).root).visibleOrder = -(1 as LONG);

    (*infoPtr).hwndNotify = (*lpcs).hwndParent;
    (*infoPtr).hwndToolTip = 0 as HWND;

    /* Determine what type of notify should be issued (sets infoPtr->bNtfUnicode) */
    TREEVIEW_NotifyFormat(infoPtr, (*infoPtr).hwndNotify, NF_REQUERY as UINT);

    if ((*infoPtr).dwStyle & (TVS_NOTOOLTIPS as DWORD)) == 0 {
        (*infoPtr).hwndToolTip = CreateWindowExW(
            0 as DWORD,
            (TOOLTIPS_CLASSW).as_ptr() as LPCWSTR,
            core::ptr::null::<WCHAR>(),
            WS_POPUP as DWORD,
            CW_USEDEFAULT as core::ffi::c_int,
            CW_USEDEFAULT as core::ffi::c_int,
            CW_USEDEFAULT as core::ffi::c_int,
            CW_USEDEFAULT as core::ffi::c_int,
            hwnd,
            0 as HMENU,
            0 as HINSTANCE,
            0 as LPVOID,
        );
    }

    /* Make sure actual scrollbar state is consistent with uInternalStatus */
    ShowScrollBar(hwnd, SB_VERT as core::ffi::c_int, FALSE as BOOL);
    ShowScrollBar(hwnd, SB_HORZ as core::ffi::c_int, FALSE as BOOL);

    OpenThemeData(hwnd, themeClass.as_ptr() as LPCWSTR);

    return 0 as LRESULT;
}

unsafe extern "C" fn TREEVIEW_Destroy(infoPtr: *mut TREEVIEW_INFO) -> LRESULT {
    /* free item data */
    TREEVIEW_RemoveTree(infoPtr);
    /* root isn't freed with other items */
    TREEVIEW_FreeItem(infoPtr, (*infoPtr).root as *mut TREEVIEW_ITEM);
    DPA_Destroy((*infoPtr).items);

    /* Restore original wndproc */
    if !(*infoPtr).hwndEdit.is_null() {
        SetWindowLongPtrW(
            (*infoPtr).hwndEdit,
            GWLP_WNDPROC,
            ::core::mem::transmute::<WNDPROC, DWORD_PTR>((*infoPtr).wpEditOrig) as LONG_PTR,
        );
    }

    CloseThemeData(GetWindowTheme((*infoPtr).hwnd));

    /* Deassociate treeview from the window before doing anything drastic. */
    SetWindowLongPtrW((*infoPtr).hwnd, 0, 0);

    DeleteObject((*infoPtr).hDefaultFont as HGDIOBJ);
    DeleteObject((*infoPtr).hBoldFont as HGDIOBJ);
    DeleteObject((*infoPtr).hUnderlineFont as HGDIOBJ);
    DeleteObject((*infoPtr).hBoldUnderlineFont as HGDIOBJ);
    DestroyWindow((*infoPtr).hwndToolTip);
    heap_free(infoPtr as *mut ::core::ffi::c_void);

    return 0;
}

unsafe extern "C" fn TREEVIEW_ScrollKeyDown(infoPtr: *mut TREEVIEW_INFO, key: WPARAM) -> LRESULT {
    static scroll: [u8; 8] = [
        ((SB_VERT as u8) << 7) | (SB_PAGEUP as u8),
        ((SB_VERT as u8) << 7) | (SB_PAGEDOWN as u8),
        ((SB_VERT as u8) << 7) | (SB_BOTTOM as u8),
        ((SB_VERT as u8) << 7) | (SB_TOP as u8),
        ((SB_HORZ as u8) << 7) | (SB_LINEUP as u8),
        ((SB_VERT as u8) << 7) | (SB_LINEUP as u8),
        ((SB_HORZ as u8) << 7) | (SB_LINEDOWN as u8),
        ((SB_VERT as u8) << 7) | (SB_LINEDOWN as u8),
    ];

    if key >= VK_PRIOR as WPARAM && key <= VK_DOWN as WPARAM {
        let code: u8 = scroll[(key.wrapping_sub(VK_PRIOR as WPARAM)) as usize];

        let f: unsafe extern "C" fn(*mut TREEVIEW_INFO, WPARAM) -> LRESULT =
            if ((code as u32) & (1 << 7))
                == (SB_HORZ << 7)
            {
                TREEVIEW_HScroll
            } else {
                TREEVIEW_VScroll
            };
        f(infoPtr, ((code as u32) & 0x7F) as WPARAM);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TREEVIEW_KeyDown(mut infoPtr: *mut TREEVIEW_INFO, mut wParam: WPARAM) -> LRESULT {
    /* If it is non-NULL and different, it will be selected and visible. */
    let mut newSelection: *mut TREEVIEW_ITEM = core::ptr::null_mut::<TREEVIEW_ITEM>();
    let mut prevItem: *mut TREEVIEW_ITEM = (*infoPtr).selectedItem as *mut TREEVIEW_ITEM;
    let mut nmkeydown: NMTVKEYDOWN = core::mem::zeroed();

    nmkeydown.wVKey = wParam as WORD;
    nmkeydown.flags = 0 as UINT;
    TREEVIEW_SendRealNotify(infoPtr, (TVN_KEYDOWN) as UINT, core::ptr::addr_of_mut!(nmkeydown.hdr));

    if prevItem.is_null() {
        return FALSE as LRESULT;
    }

    if (GetAsyncKeyState(VK_CONTROL as core::ffi::c_int) as core::ffi::c_int & 0x8000 as core::ffi::c_int) != 0 {
        return TREEVIEW_ScrollKeyDown(infoPtr, wParam);
    }

    match wParam as UINT {
        VK_UP => {
            newSelection = TREEVIEW_GetPrevListItem(infoPtr, prevItem);
            if newSelection.is_null() {
                newSelection = (*(*infoPtr).root).firstChild as *mut TREEVIEW_ITEM;
            }
        }

        VK_DOWN => {
            newSelection = TREEVIEW_GetNextListItem(infoPtr, prevItem);
        }

        VK_RETURN => {
            TREEVIEW_SendSimpleNotify(infoPtr, NM_RETURN);
        }

        VK_HOME => {
            newSelection = (*(*infoPtr).root).firstChild as *mut TREEVIEW_ITEM;
        }

        VK_END => {
            newSelection = TREEVIEW_GetLastListItem(infoPtr, (*infoPtr).root as *mut TREEVIEW_ITEM);
        }

        VK_LEFT => {
            if (*prevItem).state & TVIS_EXPANDED as UINT != 0 {
                TREEVIEW_Collapse(
                    infoPtr,
                    prevItem,
                    FALSE as BOOL,
                    TRUE as BOOL,
                );
            } else if (*prevItem).parent != (*infoPtr).root {
                newSelection = (*prevItem).parent as *mut TREEVIEW_ITEM;
            }
        }

        VK_RIGHT => {
            if TREEVIEW_HasChildren(infoPtr, prevItem) != 0 {
                if (*prevItem).state & TVIS_EXPANDED as UINT == 0 {
                    TREEVIEW_Expand(
                        infoPtr,
                        prevItem,
                        FALSE as BOOL,
                        TRUE as BOOL,
                    );
                } else {
                    newSelection = (*prevItem).firstChild as *mut TREEVIEW_ITEM;
                }
            }
        }

        VK_MULTIPLY => {
            TREEVIEW_ExpandAll(infoPtr, prevItem);
        }

        VK_ADD => {
            TREEVIEW_Expand(
                infoPtr,
                prevItem,
                FALSE as BOOL,
                TRUE as BOOL,
            );
        }

        VK_SUBTRACT => {
            TREEVIEW_Collapse(
                infoPtr,
                prevItem,
                FALSE as BOOL,
                TRUE as BOOL,
            );
        }

        VK_PRIOR => {
            newSelection = TREEVIEW_GetListItem(
                infoPtr,
                prevItem,
                -(TREEVIEW_GetVisibleCount(infoPtr) as LONG),
            );
        }

        VK_NEXT => {
            newSelection = TREEVIEW_GetListItem(
                infoPtr,
                prevItem,
                TREEVIEW_GetVisibleCount(infoPtr) as LONG,
            );
        }

        VK_BACK => {
            newSelection = (*prevItem).parent as *mut TREEVIEW_ITEM;
            if newSelection == (*infoPtr).root as *mut TREEVIEW_ITEM {
                newSelection = core::ptr::null_mut::<TREEVIEW_ITEM>();
            }
        }

        VK_SPACE => {
            if ((*infoPtr).dwStyle & (TVS_CHECKBOXES as DWORD)) != 0 {
                TREEVIEW_ToggleItemState(infoPtr, prevItem);
            }
        }

        _ => {}
    }

    if !newSelection.is_null() && newSelection != prevItem {
        if TREEVIEW_DoSelectItem(
            infoPtr,
            TVGN_CARET as INT,
            newSelection as HTREEITEM,
            TVC_BYKEYBOARD as INT,
        ) != 0
        {
            TREEVIEW_EnsureVisible(infoPtr, newSelection as HTREEITEM, FALSE as BOOL);
        }
    }

    return FALSE as LRESULT;
}

unsafe extern "C" fn TREEVIEW_MouseLeave(infoPtr: *mut TREEVIEW_INFO) -> LRESULT {
    TREEVIEW_InvalidateItem(infoPtr as *const TREEVIEW_INFO, (*infoPtr).hotItem as *const TREEVIEW_ITEM);
    (*infoPtr).hotItem = core::ptr::null_mut();

    return 0;
}

unsafe extern "C" fn TREEVIEW_MouseMove(
    mut infoPtr: *mut TREEVIEW_INFO,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut trackinfo: TRACKMOUSEEVENT = core::mem::zeroed();
    let mut item: *mut TREEVIEW_ITEM = core::ptr::null_mut::<TREEVIEW_ITEM>();
    let mut ht: TVHITTESTINFO = core::mem::zeroed();
    let mut item_hit: BOOL = 0;
    if (*infoPtr).dwStyle & TVS_TRACKSELECT as DWORD == 0 {
        return 0 as LRESULT;
    }
    /* fill in the TRACKMOUSEEVENT struct */
    trackinfo.cbSize = core::mem::size_of::<TRACKMOUSEEVENT>() as DWORD;
    trackinfo.dwFlags = TME_QUERY as DWORD;
    trackinfo.hwndTrack = (*infoPtr).hwnd;
    /* call _TrackMouseEvent to see if we are currently tracking for this hwnd */
    _TrackMouseEvent(core::ptr::addr_of_mut!(trackinfo));
    /* Make sure tracking is enabled so we receive a WM_MOUSELEAVE message */
    if trackinfo.dwFlags & TME_LEAVE as DWORD == 0 {
        trackinfo.dwFlags = TME_LEAVE as DWORD; /* notify upon leaving */
        trackinfo.hwndTrack = (*infoPtr).hwnd;
        /* do it as fast as possible, minimal systimer latency will be used */
        trackinfo.dwHoverTime = 1 as DWORD;
        /* call TRACKMOUSEEVENT so we receive a WM_MOUSELEAVE message */
        /* and can properly deactivate the hot item */
        _TrackMouseEvent(core::ptr::addr_of_mut!(trackinfo));
    }
    ht.pt.x = LOWORD!(lParam) as ::core::ffi::c_short as LONG;
    ht.pt.y = HIWORD!(lParam) as ::core::ffi::c_short as LONG;

    item = TREEVIEW_HitTest(infoPtr, core::ptr::addr_of_mut!(ht));
    item_hit = TREEVIEW_IsItemHit(infoPtr, core::ptr::addr_of_mut!(ht));
    if item != ((*infoPtr).hotItem).cast() || item_hit == 0 {
        /* redraw old hot item */
        TREEVIEW_InvalidateItem(infoPtr, (*infoPtr).hotItem as *const TREEVIEW_ITEM);
        (*infoPtr).hotItem = core::ptr::null_mut::<_TREEITEM>();
        if !item.is_null() && item_hit != 0 {
            (*infoPtr).hotItem = item as HTREEITEM;
            /* redraw new hot item */
            TREEVIEW_InvalidateItem(infoPtr, (*infoPtr).hotItem as *const TREEVIEW_ITEM);
        }
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TREEVIEW_NCPaint(infoPtr: *const TREEVIEW_INFO, region: HRGN, lParam: LPARAM) -> BOOL {
    let theme: HTHEME = GetWindowTheme((*infoPtr).hwnd);
    let mut dc: HDC;
    let mut r: RECT = core::mem::zeroed();
    let mut cliprgn: HRGN;
    let cxEdge: i32 = GetSystemMetrics(SM_CXEDGE as i32);
    let cyEdge: i32 = GetSystemMetrics(SM_CYEDGE as i32);

    if theme.is_null() {
        return DefWindowProcW(
            (*infoPtr).hwnd,
            WM_NCPAINT as UINT,
            region as WPARAM,
            lParam,
        ) as BOOL;
    }

    GetWindowRect((*infoPtr).hwnd, &mut r);

    cliprgn = CreateRectRgn(
        r.left + cxEdge,
        r.top + cyEdge,
        r.right - cxEdge,
        r.bottom - cyEdge,
    );
    if region != 1 as HRGN {
        CombineRgn(cliprgn, cliprgn, region, RGN_AND as i32);
    }
    OffsetRect(&mut r, -r.left, -r.top);

    dc = GetWindowDC((*infoPtr).hwnd);
    /* Exclude client part */
    ExcludeClipRect(
        dc,
        r.left + cxEdge,
        r.top + cyEdge,
        r.right - cxEdge,
        r.bottom - cyEdge,
    );

    if IsThemeBackgroundPartiallyTransparent(
        theme,
        0 as i32,
        0 as i32,
    ) != 0
    {
        DrawThemeParentBackground((*infoPtr).hwnd, dc, &mut r);
    }
    DrawThemeBackground(
        theme,
        dc,
        0 as i32,
        0 as i32,
        &mut r,
        core::ptr::null(),
    );
    ReleaseDC((*infoPtr).hwnd, dc);

    /* Call default proc to get the scrollbars etc. painted */
    DefWindowProcW(
        (*infoPtr).hwnd,
        WM_NCPAINT as UINT,
        cliprgn as WPARAM,
        0 as LPARAM,
    );
    DeleteObject((cliprgn).cast());

    return TRUE as BOOL;
}

unsafe extern "C" fn TREEVIEW_Notify(
    mut infoPtr: *const TREEVIEW_INFO,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut lpnmh: LPNMHDR = lParam as LPNMHDR;
    if (*lpnmh).code == (PGN_CALCSIZE) as u32 {
        let mut lppgc: LPNMPGCALCSIZE = lParam as LPNMPGCALCSIZE;
        if (*lppgc).dwFlag == PGF_CALCWIDTH {
            (*lppgc).iWidth = (*infoPtr).treeWidth as ::core::ffi::c_int;
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/treeview.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TREEVIEW_Notify\0" as *const u8 as *const ::core::ffi::c_char,
                    5514 as ::core::ffi::c_int,
                    b"got PGN_CALCSIZE, returning horz size = %d, client=%d\n\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    (*infoPtr).treeWidth,
                    (*infoPtr).clientWidth,
                );
            }
        } else {
            (*lppgc).iHeight = (*infoPtr).treeHeight as ::core::ffi::c_int;
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/treeview.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TREEVIEW_Notify\0" as *const u8 as *const ::core::ffi::c_char,
                    5519 as ::core::ffi::c_int,
                    b"got PGN_CALCSIZE, returning vert size = %d, client=%d\n\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    (*infoPtr).treeHeight,
                    (*infoPtr).clientHeight,
                );
            }
        }
        return 0 as LRESULT;
    }
    return DefWindowProcW((*infoPtr).hwnd, WM_NOTIFY as UINT, wParam, lParam);
}

unsafe extern "C" fn TREEVIEW_Size(infoPtr: *mut TREEVIEW_INFO, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    if wParam == (SIZE_RESTORED) as _ {
        (*infoPtr).clientWidth = LOWORD!(lParam as DWORD_PTR) as i16 as LONG;
        (*infoPtr).clientHeight = HIWORD!(lParam as DWORD_PTR) as i16 as LONG;

        TREEVIEW_RecalculateVisibleOrder(infoPtr, ::core::ptr::null_mut::<TREEVIEW_ITEM>());
        TREEVIEW_SetFirstVisible(infoPtr, (*infoPtr).firstVisible as *mut TREEVIEW_ITEM, TRUE);
        TREEVIEW_UpdateScrollBars(infoPtr);
    } else {
    }

    TREEVIEW_Invalidate(infoPtr, ::core::ptr::null::<TREEVIEW_ITEM>());
    return 0;
}

unsafe extern "C" fn TREEVIEW_StyleChanged(infoPtr: *mut TREEVIEW_INFO, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    if wParam == GWL_STYLE as WPARAM {
        let mut dwNewStyle: DWORD = (*(lParam as LPSTYLESTRUCT)).styleNew;
        if (((*infoPtr).dwStyle ^ dwNewStyle) & (TVS_CHECKBOXES as DWORD)) != 0 {
            if (dwNewStyle & (TVS_CHECKBOXES as DWORD)) != 0 {
                TREEVIEW_InitCheckboxes(infoPtr);

                /* set all items to state image index 1 */
                TREEVIEW_ResetImageStateIndex(infoPtr, (*infoPtr).root as *mut TREEVIEW_ITEM);
            } else {
            }
        }

        if (((*infoPtr).dwStyle ^ dwNewStyle) & (TVS_NOTOOLTIPS as DWORD)) != 0 {
            if ((*infoPtr).dwStyle & (TVS_NOTOOLTIPS as DWORD)) != 0 {
                (*infoPtr).hwndToolTip = COMCTL32_CreateToolTip((*infoPtr).hwnd);
            } else {
                DestroyWindow((*infoPtr).hwndToolTip);
                (*infoPtr).hwndToolTip = core::ptr::null_mut();
            }
        }

        (*infoPtr).dwStyle = dwNewStyle;
    }

    TREEVIEW_EndEditLabelNow(infoPtr, TRUE as BOOL);
    TREEVIEW_UpdateSubTree(infoPtr, (*infoPtr).root as *mut TREEVIEW_ITEM);
    TREEVIEW_UpdateScrollBars(infoPtr);
    TREEVIEW_Invalidate(infoPtr, core::ptr::null::<TREEVIEW_ITEM>());

    return 0 as LRESULT;
}

unsafe extern "C" fn TREEVIEW_SetCursor(
    infoPtr: *const TREEVIEW_INFO,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut item: *mut TREEVIEW_ITEM = core::ptr::null_mut();
    let mut ht: TVHITTESTINFO = core::mem::zeroed();
    let mut nmmouse: NMMOUSE = core::mem::zeroed();

    GetCursorPos(core::ptr::addr_of_mut!(ht.pt));
    ScreenToClient((*infoPtr).hwnd, core::ptr::addr_of_mut!(ht.pt));

    item = TREEVIEW_HitTest(infoPtr, core::ptr::addr_of_mut!(ht));

    memset(
        core::ptr::addr_of_mut!(nmmouse) as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<NMMOUSE>() as size_t,
    );
    if !item.is_null() {
        nmmouse.dwItemSpec = item as DWORD_PTR;
        nmmouse.dwItemData = (*item).lParam as DWORD_PTR;
    }
    nmmouse.pt.x = 0 as LONG;
    nmmouse.pt.y = 0 as LONG;
    nmmouse.dwHitInfo = (lParam as DWORD) as _;
    if TREEVIEW_SendRealNotify(
        infoPtr,
        NM_SETCURSOR as UINT,
        core::ptr::addr_of_mut!(nmmouse.hdr),
    ) != 0
    {
        return 0 as LRESULT;
    }

    if !item.is_null()
        && ((*infoPtr).dwStyle & (TVS_TRACKSELECT as DWORD)) != 0
        && TREEVIEW_IsItemHit(infoPtr, core::ptr::addr_of_mut!(ht)) != 0
    {
        SetCursor((*infoPtr).hcurHand);
        return 0 as LRESULT;
    } else {
        return DefWindowProcW(
            (*infoPtr).hwnd,
            WM_SETCURSOR as UINT,
            wParam,
            lParam,
        );
    }
}

unsafe extern "C" fn TREEVIEW_SetFocus(infoPtr: *mut TREEVIEW_INFO) -> LRESULT {
    if (*infoPtr).selectedItem.is_null() {
        TREEVIEW_DoSelectItem(infoPtr, (TVGN_CARET) as INT, (*infoPtr).firstVisible, (TVC_UNKNOWN) as INT);
    }

    TREEVIEW_Invalidate(infoPtr, (*infoPtr).selectedItem as *const TREEVIEW_ITEM);
    TREEVIEW_SendSimpleNotify(infoPtr, NM_SETFOCUS);
    return 0;
}

unsafe extern "C" fn TREEVIEW_KillFocus(infoPtr: *const TREEVIEW_INFO) -> LRESULT {
    TREEVIEW_Invalidate(infoPtr, (*infoPtr).selectedItem as *const TREEVIEW_ITEM);
    UpdateWindow((*infoPtr).hwnd);
    TREEVIEW_SendSimpleNotify(infoPtr, NM_KILLFOCUS);
    return 0;
}

unsafe extern "C" fn TREEVIEW_ThemeChanged(infoPtr: *const TREEVIEW_INFO) -> LRESULT {
    let theme: HTHEME = GetWindowTheme((*infoPtr).hwnd);
    CloseThemeData(theme);
    OpenThemeData((*infoPtr).hwnd, core::ptr::addr_of!(themeClass) as LPCWSTR);
    return 0;
}

unsafe extern "system" fn TREEVIEW_WindowProc(
    mut hwnd: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut infoPtr: *mut TREEVIEW_INFO = TREEVIEW_GetInfoPtr(hwnd);

    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/comctl32/treeview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TREEVIEW_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            5673 as ::core::ffi::c_int,
            b"hwnd %p msg %04x wp=%08lx lp=%08lx\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hwnd,
            uMsg,
            wParam,
            lParam,
        );
    }

    'def: {
        if !infoPtr.is_null() {
            TREEVIEW_VerifyTree(infoPtr);
        } else {
            if uMsg == WM_CREATE {
                TREEVIEW_Create(hwnd, lParam as LPCREATESTRUCTW as *const CREATESTRUCTW);
            } else {
                break 'def;
            }
        }

        match uMsg {
            TVM_CREATEDRAGIMAGE => return TREEVIEW_CreateDragImage(infoPtr, lParam),
            TVM_DELETEITEM => {
                return TREEVIEW_DeleteItem(infoPtr, lParam as HTREEITEM);
            }
            TVM_EDITLABELA | TVM_EDITLABELW => {
                return TREEVIEW_EditLabel(infoPtr, lParam as HTREEITEM) as LRESULT;
            }
            TVM_ENDEDITLABELNOW => return TREEVIEW_EndEditLabelNow(infoPtr, wParam as BOOL),
            TVM_ENSUREVISIBLE => {
                return TREEVIEW_EnsureVisible(
                    infoPtr,
                    lParam as HTREEITEM,
                    TRUE,
                );
            }
            TVM_EXPAND => {
                return TREEVIEW_ExpandMsg(infoPtr, wParam as UINT, lParam as HTREEITEM);
            }
            TVM_GETBKCOLOR => return TREEVIEW_GetBkColor(infoPtr),
            TVM_GETCOUNT => return TREEVIEW_GetCount(infoPtr),
            TVM_GETEDITCONTROL => return TREEVIEW_GetEditControl(infoPtr),
            TVM_GETIMAGELIST => return TREEVIEW_GetImageList(infoPtr, wParam),
            TVM_GETINDENT => return TREEVIEW_GetIndent(infoPtr),
            TVM_GETINSERTMARKCOLOR => return TREEVIEW_GetInsertMarkColor(infoPtr),
            TVM_GETISEARCHSTRINGA => {
                if ((*__wine_dbch___default).flags as i32
                    & ((1i32) << (__WINE_DBCL_FIXME as i32)))
                    != 0
                {
                    ros_dbg_log(
                        __WINE_DBCL_FIXME,
                        __wine_dbch___default,
                        b"dll/win32/comctl32/treeview.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"TREEVIEW_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                        5724 as ::core::ffi::c_int,
                        b"Unimplemented msg TVM_GETISEARCHSTRINGA\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                }
                return 0 as LRESULT;
            }
            TVM_GETISEARCHSTRINGW => {
                if ((*__wine_dbch___default).flags as i32
                    & ((1i32) << (__WINE_DBCL_FIXME as i32)))
                    != 0
                {
                    ros_dbg_log(
                        __WINE_DBCL_FIXME,
                        __wine_dbch___default,
                        b"dll/win32/comctl32/treeview.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"TREEVIEW_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                        5728 as ::core::ffi::c_int,
                        b"Unimplemented msg TVM_GETISEARCHSTRINGW\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                }
                return 0 as LRESULT;
            }
            TVM_GETITEMA | TVM_GETITEMW => {
                return TREEVIEW_GetItemT(
                    infoPtr,
                    lParam as LPTVITEMEXW,
                    (uMsg == TVM_GETITEMW) as BOOL,
                );
            }
            TVM_GETITEMHEIGHT => return TREEVIEW_GetItemHeight(infoPtr),
            TVM_GETITEMRECT => {
                return TREEVIEW_GetItemRect(infoPtr, wParam as BOOL, lParam as LPRECT);
            }
            TVM_GETITEMSTATE => {
                return TREEVIEW_GetItemState(infoPtr, wParam as HTREEITEM, lParam as UINT);
            }
            TVM_GETLINECOLOR => return TREEVIEW_GetLineColor(infoPtr),
            TVM_GETNEXTITEM => {
                return TREEVIEW_GetNextItem(infoPtr, wParam as UINT, lParam as HTREEITEM);
            }
            TVM_GETSCROLLTIME => return TREEVIEW_GetScrollTime(infoPtr),
            TVM_GETTEXTCOLOR => return TREEVIEW_GetTextColor(infoPtr),
            TVM_GETTOOLTIPS => return TREEVIEW_GetToolTips(infoPtr),
            TVM_GETUNICODEFORMAT => return TREEVIEW_GetUnicodeFormat(infoPtr),
            TVM_GETVISIBLECOUNT => return TREEVIEW_GetVisibleCount(infoPtr),
            TVM_HITTEST => {
                return TREEVIEW_HitTest(infoPtr, lParam as LPTVHITTESTINFO) as LRESULT;
            }
            TVM_INSERTITEMA | TVM_INSERTITEMW => {
                return TREEVIEW_InsertItemT(
                    infoPtr,
                    lParam as LPTVINSERTSTRUCTW as *const TVINSERTSTRUCTW,
                    (uMsg == TVM_INSERTITEMW) as BOOL,
                );
            }
            TVM_SELECTITEM => {
                return TREEVIEW_SelectItem(infoPtr, wParam as INT, lParam as HTREEITEM);
            }
            TVM_SETBKCOLOR => {
                return TREEVIEW_SetBkColor(infoPtr, lParam as COLORREF);
            }
            TVM_SETIMAGELIST => {
                return TREEVIEW_SetImageList(infoPtr, (wParam) as UINT, lParam as HIMAGELIST);
            }
            TVM_SETINDENT => {
                return TREEVIEW_SetIndent(infoPtr, wParam as UINT);
            }
            TVM_SETINSERTMARK => {
                return TREEVIEW_SetInsertMark(infoPtr, wParam as BOOL, lParam as HTREEITEM);
            }
            TVM_SETINSERTMARKCOLOR => {
                return TREEVIEW_SetInsertMarkColor(infoPtr, lParam as COLORREF);
            }
            TVM_SETITEMA | TVM_SETITEMW => {
                return TREEVIEW_SetItemT(
                    infoPtr,
                    lParam as LPTVITEMEXW as *const TVITEMEXW,
                    (uMsg == TVM_SETITEMW) as BOOL,
                );
            }
            TVM_SETLINECOLOR => {
                return TREEVIEW_SetLineColor(infoPtr, lParam as COLORREF);
            }
            TVM_SETITEMHEIGHT => {
                return TREEVIEW_SetItemHeight(infoPtr, wParam as SHORT as INT);
            }
            TVM_SETSCROLLTIME => {
                return TREEVIEW_SetScrollTime(infoPtr, wParam as UINT);
            }
            TVM_SETTEXTCOLOR => {
                return TREEVIEW_SetTextColor(infoPtr, lParam as COLORREF);
            }
            TVM_SETTOOLTIPS => {
                return TREEVIEW_SetToolTips(infoPtr, wParam as HWND);
            }
            TVM_SETUNICODEFORMAT => return TREEVIEW_SetUnicodeFormat(infoPtr, wParam as BOOL),
            TVM_SORTCHILDREN => return TREEVIEW_SortChildren(infoPtr, lParam),
            TVM_SORTCHILDRENCB => {
                return TREEVIEW_SortChildrenCB(infoPtr, lParam as LPTVSORTCB);
            }
            WM_CHAR => return TREEVIEW_ProcessLetterKeys(infoPtr, wParam, lParam) as LRESULT,
            WM_COMMAND => return TREEVIEW_Command(infoPtr, wParam, lParam),
            WM_DESTROY => return TREEVIEW_Destroy(infoPtr),
            WM_ERASEBKGND => {
                return TREEVIEW_EraseBackground(infoPtr, wParam as HDC);
            }
            WM_GETDLGCODE => {
                return (DLGC_WANTARROWS | DLGC_WANTCHARS) as LRESULT;
            }
            WM_GETFONT => return TREEVIEW_GetFont(infoPtr),
            WM_HSCROLL => return TREEVIEW_HScroll(infoPtr, wParam),
            WM_KEYDOWN => return TREEVIEW_KeyDown(infoPtr, wParam),
            WM_KILLFOCUS => return TREEVIEW_KillFocus(infoPtr),
            WM_LBUTTONDBLCLK => return TREEVIEW_LButtonDoubleClick(infoPtr, lParam),
            WM_LBUTTONDOWN => return TREEVIEW_LButtonDown(infoPtr, lParam),
            WM_MOUSELEAVE => return TREEVIEW_MouseLeave(infoPtr),
            WM_MOUSEMOVE => return TREEVIEW_MouseMove(infoPtr, lParam),
            WM_NCLBUTTONDOWN => {
                if !(*infoPtr).hwndEdit.is_null() {
                    SetFocus((*infoPtr).hwnd);
                }
                break 'def;
            }
            WM_NCPAINT => {
                return TREEVIEW_NCPaint(infoPtr, wParam as HRGN, lParam) as LRESULT;
            }
            WM_NOTIFY => return TREEVIEW_Notify(infoPtr, wParam, lParam),
            WM_NOTIFYFORMAT => {
                return TREEVIEW_NotifyFormat(infoPtr, wParam as HWND, lParam as UINT) as LRESULT;
            }
            WM_PRINTCLIENT => {
                return TREEVIEW_PrintClient(infoPtr, wParam as HDC, lParam as DWORD);
            }
            WM_PAINT => {
                return TREEVIEW_Paint(infoPtr, wParam as HDC);
            }
            WM_RBUTTONDOWN => return TREEVIEW_RButtonDown(infoPtr, lParam),
            WM_SETCURSOR => return TREEVIEW_SetCursor(infoPtr, wParam, lParam),
            WM_SETFOCUS => return TREEVIEW_SetFocus(infoPtr),
            WM_SETFONT => {
                return TREEVIEW_SetFont(infoPtr, wParam as HFONT, lParam as BOOL);
            }
            WM_SETREDRAW => return TREEVIEW_SetRedraw(infoPtr, wParam),
            WM_SIZE => return TREEVIEW_Size(infoPtr, wParam, lParam),
            WM_STYLECHANGED => return TREEVIEW_StyleChanged(infoPtr, wParam, lParam),
            WM_SYSCOLORCHANGE => {
                COMCTL32_RefreshSysColors();
                return 0 as LRESULT;
            }
            WM_TIMER => return TREEVIEW_HandleTimer(infoPtr, wParam),
            WM_THEMECHANGED => return TREEVIEW_ThemeChanged(infoPtr),
            WM_VSCROLL => return TREEVIEW_VScroll(infoPtr, wParam),
            WM_MOUSEWHEEL => return TREEVIEW_MouseWheel(infoPtr, wParam, lParam),
            WM_DRAWITEM => {
                if ((*__wine_dbch___default).flags as i32
                    & ((1i32) << (__WINE_DBCL_TRACE as i32)))
                    != 0
                {
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __wine_dbch___default,
                        b"dll/win32/comctl32/treeview.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"TREEVIEW_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                        5924 as ::core::ffi::c_int,
                        b"drawItem\n\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                break 'def;
            }
            _ => {
                if uMsg >= WM_USER
                    && uMsg < WM_APP
                    && COMCTL32_IsReflectedMessage(uMsg) == 0
                {
                    if ((*__wine_dbch___default).flags as i32
                        & ((1i32) << (__WINE_DBCL_TRACE as i32)))
                        != 0
                    {
                        ros_dbg_log(
                            __WINE_DBCL_TRACE,
                            __wine_dbch___default,
                            b"dll/win32/comctl32/treeview.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"TREEVIEW_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                            5930 as ::core::ffi::c_int,
                            b"Unknown msg %04x wp=%08lx lp=%08lx\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                            uMsg,
                            wParam,
                            lParam,
                        );
                    }
                }
            }
        }
    }

    DefWindowProcW(hwnd, uMsg, wParam, lParam)
}
#[no_mangle]
pub unsafe extern "C" fn TREEVIEW_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(wndClass) as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSW>() as usize,
    );
    wndClass.style = (CS_GLOBALCLASS | CS_DBLCLKS) as UINT;
    wndClass.lpfnWndProc =
        Some(TREEVIEW_WindowProc);
    wndClass.cbClsExtra = 0 as i32;
    wndClass.cbWndExtra = core::mem::size_of::<TREEVIEW_INFO>() as i32;

    wndClass.hCursor = LoadCursorW(
        0 as HINSTANCE,
        IDC_ARROW as LPWSTR,
    );
    wndClass.hbrBackground = 0 as HBRUSH;
    wndClass.lpszClassName = (WC_TREEVIEWW).as_ptr() as LPCWSTR;

    RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn TREEVIEW_Unregister() {
    UnregisterClassW((WC_TREEVIEWW).as_ptr() as *const u16, core::ptr::null_mut());
}

unsafe extern "C" fn TREEVIEW_VerifyItemCommon(infoPtr: *mut TREEVIEW_INFO, item: *const TREEVIEW_ITEM) {
    if infoPtr.is_null() {
        _assert(
            b"infoPtr != NULL\0".as_ptr() as *const i8,
            b"dll/win32/comctl32/treeview.c\0".as_ptr() as *const i8,
            5975,
        );
    }
    if item.is_null() {
        _assert(
            b"item != NULL\0".as_ptr() as *const i8,
            b"dll/win32/comctl32/treeview.c\0".as_ptr() as *const i8,
            5976,
        );
    }
    if !(((*item).firstChild.is_null()) == ((*item).lastChild.is_null())) {
        _assert(
            b"(item->firstChild == NULL) == (item->lastChild == NULL)\0".as_ptr() as *const i8,
            b"dll/win32/comctl32/treeview.c\0".as_ptr() as *const i8,
            5979,
        );
    }
    if (*item).firstChild as *const TREEVIEW_ITEM == item {
        _assert(
            b"item->firstChild != item\0".as_ptr() as *const i8,
            b"dll/win32/comctl32/treeview.c\0".as_ptr() as *const i8,
            5981,
        );
    }
    if (*item).lastChild as *const TREEVIEW_ITEM == item {
        _assert(
            b"item->lastChild != item\0".as_ptr() as *const i8,
            b"dll/win32/comctl32/treeview.c\0".as_ptr() as *const i8,
            5982,
        );
    }
    if !(*item).firstChild.is_null() {
        if !((*(*item).firstChild).parent as *const TREEVIEW_ITEM == item) {
            _assert(
                b"item->firstChild->parent == item\0".as_ptr() as *const i8,
                b"dll/win32/comctl32/treeview.c\0".as_ptr() as *const i8,
                5986,
            );
        }
        if !(*(*item).firstChild).prevSibling.is_null() {
            _assert(
                b"item->firstChild->prevSibling == NULL\0".as_ptr() as *const i8,
                b"dll/win32/comctl32/treeview.c\0".as_ptr() as *const i8,
                5987,
            );
        }
    }
    if !(*item).lastChild.is_null() {
        if !((*(*item).lastChild).parent as *const TREEVIEW_ITEM == item) {
            _assert(
                b"item->lastChild->parent == item\0".as_ptr() as *const i8,
                b"dll/win32/comctl32/treeview.c\0".as_ptr() as *const i8,
                5992,
            );
        }
        if !(*(*item).lastChild).nextSibling.is_null() {
            _assert(
                b"item->lastChild->nextSibling == NULL\0".as_ptr() as *const i8,
                b"dll/win32/comctl32/treeview.c\0".as_ptr() as *const i8,
                5993,
            );
        }
    }
    if (*item).nextSibling as *const TREEVIEW_ITEM == item {
        _assert(
            b"item->nextSibling != item\0".as_ptr() as *const i8,
            b"dll/win32/comctl32/treeview.c\0".as_ptr() as *const i8,
            5996,
        );
    }
    if !(*item).nextSibling.is_null() {
        if !((*(*item).nextSibling).parent == (*item).parent) {
            _assert(
                b"item->nextSibling->parent == item->parent\0".as_ptr() as *const i8,
                b"dll/win32/comctl32/treeview.c\0".as_ptr() as *const i8,
                5999,
            );
        }
        if !((*(*item).nextSibling).prevSibling as *const TREEVIEW_ITEM == item) {
            _assert(
                b"item->nextSibling->prevSibling == item\0".as_ptr() as *const i8,
                b"dll/win32/comctl32/treeview.c\0".as_ptr() as *const i8,
                6000,
            );
        }
    }
    if (*item).prevSibling as *const TREEVIEW_ITEM == item {
        _assert(
            b"item->prevSibling != item\0".as_ptr() as *const i8,
            b"dll/win32/comctl32/treeview.c\0".as_ptr() as *const i8,
            6003,
        );
    }
    if !(*item).prevSibling.is_null() {
        if !((*(*item).prevSibling).parent == (*item).parent) {
            _assert(
                b"item->prevSibling->parent == item->parent\0".as_ptr() as *const i8,
                b"dll/win32/comctl32/treeview.c\0".as_ptr() as *const i8,
                6006,
            );
        }
        if !((*(*item).prevSibling).nextSibling as *const TREEVIEW_ITEM == item) {
            _assert(
                b"item->prevSibling->nextSibling == item\0".as_ptr() as *const i8,
                b"dll/win32/comctl32/treeview.c\0".as_ptr() as *const i8,
                6007,
            );
        }
    }
}

unsafe extern "C" fn TREEVIEW_VerifyItem(infoPtr: *mut TREEVIEW_INFO, item: *const TREEVIEW_ITEM) {
    assert!((!item.is_null()) as i32);
    assert!((!(*item).parent.is_null()) as i32);
    assert!(((*item).parent != item as HTREEITEM) as i32);
    assert!(((*item).iLevel == (*(*item).parent).iLevel + 1) as i32);
    assert!((DPA_GetPtrIndex((*infoPtr).items, item as *const core::ffi::c_void) != -1) as i32);
    TREEVIEW_VerifyItemCommon(infoPtr, item);
    TREEVIEW_VerifyChildren(infoPtr, item);
}

unsafe extern "C" fn TREEVIEW_VerifyChildren(infoPtr: *mut TREEVIEW_INFO, item: *const TREEVIEW_ITEM) {
    let mut child: *const TREEVIEW_ITEM = core::ptr::null::<TREEVIEW_ITEM>();
    assert!((!item.is_null()) as i32);
    child = (*item).firstChild as *const TREEVIEW_ITEM;
    while !child.is_null() {
        TREEVIEW_VerifyItem(infoPtr, child);
        child = (*child).nextSibling as *const TREEVIEW_ITEM;
    }
}

unsafe extern "C" fn TREEVIEW_VerifyRoot(infoPtr: *mut TREEVIEW_INFO) {
    let root: *mut TREEVIEW_ITEM = (*infoPtr).root as *mut TREEVIEW_ITEM;

    assert!((!root.is_null()) as core::ffi::c_int);
    assert!(((*root).iLevel == -(1)) as core::ffi::c_int);
    assert!(((*root).parent.is_null()) as core::ffi::c_int);
    assert!(((*root).prevSibling.is_null()) as core::ffi::c_int);

    TREEVIEW_VerifyItemCommon(infoPtr, root);

    TREEVIEW_VerifyChildren(infoPtr, root);
}

unsafe extern "C" fn TREEVIEW_VerifyTree(infoPtr: *mut TREEVIEW_INFO) {
    if !TRACE_ON!(treeview) {
        return;
    }

    assert!((!infoPtr.is_null()) as i32);
    TREEVIEW_VerifyRoot(infoPtr);
}
