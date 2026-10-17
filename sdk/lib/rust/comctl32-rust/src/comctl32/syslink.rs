use crate::comctl32::comctl32undoc::Alloc;
use crate::comctl32::commctrl::COMCTL32_IsReflectedMessage;
use crate::comctl32::commctrl::COMCTL32_RefreshSysColors;
use crate::comctl32::comctl32undoc::Free;
use crate::comctl32::comctl32undoc::ReAlloc;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_syslink);
extern "system" {
    fn SetFocus(hWnd: HWND) -> HWND;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const WC_LINK: [::core::ffi::c_ushort; 8] = unsafe {
    ::core::mem::transmute::<[u8; 16], [::core::ffi::c_ushort; 8]>(*b"S\0y\0s\0L\0i\0n\0k\0\0\0")
};

pub const LIF_ITEMINDEX: u32 = 0x00000001; // commctrl
pub const LIF_STATE: u32 = 0x00000002; // commctrl
pub const LIF_ITEMID: u32 = 0x00000004; // commctrl
pub const LIF_URL: u32 = 0x00000008; // commctrl
pub const LIS_FOCUSED: u32 = 0x00000001; // commctrl
pub const LIS_ENABLED: u32 = 0x00000002; // commctrl
pub const LIS_VISITED: u32 = 0x00000004; // commctrl


#[repr(C)]
pub struct DOC_TEXTBLOCK {
    pub nChars: i32,
    pub nSkip: i32,
    pub rc: RECT,
}
pub type PDOC_TEXTBLOCK = *mut DOC_TEXTBLOCK;

pub const LIF_FLAGSMASK: u32 = LIF_STATE | LIF_ITEMID | LIF_URL;

pub const LIS_MASK: u32 = LIS_FOCUSED | LIS_ENABLED | LIS_VISITED;

pub type SL_ITEM_TYPE = i32;
pub const slText: SL_ITEM_TYPE = 0;
pub const slLink: SL_ITEM_TYPE = 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DOC_ITEM_u_Link {
    pub state: u32,
    pub szID: *mut WCHAR,
    pub szUrl: *mut WCHAR,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DOC_ITEM_u_Text {
    pub Dummy: u32,
}

#[repr(C)]
pub union DOC_ITEM_u {
    // Plain C data, so Copy fields: `ManuallyDrop` made every access an autoref through Deref.
    pub Link: DOC_ITEM_u_Link,
    pub Text: DOC_ITEM_u_Text,
}

#[repr(C)]
pub struct _DOC_ITEM {
    pub entry: _LIST_ENTRY,
    pub nText: u32,
    pub Type: SL_ITEM_TYPE,
    pub Blocks: PDOC_TEXTBLOCK,
    pub u: DOC_ITEM_u,
    pub Text: [WCHAR; 1],
}
pub type DOC_ITEM = _DOC_ITEM;
pub type PDOC_ITEM = *mut _DOC_ITEM;

#[repr(C)]
pub struct SYSLINK_INFO {
    pub Self_: HWND,
    pub Notify: HWND,
    pub Style: DWORD,
    pub Items: _LIST_ENTRY,
    pub HasFocus: BOOL,
    pub MouseDownID: i32,
    pub Font: HFONT,
    pub LinkFont: HFONT,
    pub TextColor: COLORREF,
    pub LinkColor: COLORREF,
    pub VisitedColor: COLORREF,
    pub BreakChar: WCHAR,
    pub IgnoreReturn: BOOL,
}

pub const SL_LEFTMARGIN: i32 = 0;

pub const SL_TOPMARGIN: i32 = 0;

pub const SL_RIGHTMARGIN: i32 = 0;

pub const SL_BOTTOMMARGIN: i32 = 0;

unsafe extern "C" fn SYSLINK_FreeDocItem(mut DocItem: PDOC_ITEM) {
    if (*DocItem).Type as ::core::ffi::c_uint == slLink as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        Free((*DocItem).u.Link.szID as LPVOID);
        Free((*DocItem).u.Link.szUrl as LPVOID);
    }

    Free((*DocItem).Blocks as LPVOID);

    /* we don't free Text because it's just a pointer to a character in the
       entire window text string */

    Free(DocItem as LPVOID);
}

unsafe extern "C" fn SYSLINK_AppendDocItem(
    mut infoPtr: *mut SYSLINK_INFO,
    mut Text: LPCWSTR,
    mut textlen: UINT,
    mut type_0: SL_ITEM_TYPE,
    mut LastItem: PDOC_ITEM,
) -> PDOC_ITEM {
    let mut Item: PDOC_ITEM = core::ptr::null_mut();
    textlen = min(textlen, lstrlenW(Text) as UINT);
    Item = Alloc(FIELD_OFFSET!(DOC_ITEM, Text[textlen as usize + 1]) as DWORD) as PDOC_ITEM;
    if Item.is_null() {
        return core::ptr::null_mut();
    }
    (*Item).nText = textlen;
    (*Item).Type = type_0;
    (*Item).Blocks = core::ptr::null_mut();
    lstrcpynW(
        &raw mut (*Item).Text as LPWSTR,
        Text,
        textlen.wrapping_add(1 as UINT) as core::ffi::c_int,
    );
    if !LastItem.is_null() {
        list_add_after((&raw mut (*LastItem).entry).cast(), (&raw mut (*Item).entry).cast());
    } else {
        list_add_tail((&raw mut (*infoPtr).Items).cast(), (&raw mut (*Item).entry).cast());
    }
    return Item;
}

unsafe extern "C" fn SYSLINK_ClearDoc(infoPtr: *mut SYSLINK_INFO) {
    let mut Item: *mut DOC_ITEM = core::ptr::null_mut::<DOC_ITEM>();
    let mut Item2: *mut DOC_ITEM = core::ptr::null_mut::<DOC_ITEM>();
    Item = ((*infoPtr).Items.Flink as *mut u8)
        .sub(core::mem::offset_of!(DOC_ITEM, entry)) as *mut DOC_ITEM;
    Item2 = ((*Item).entry.Flink as *mut u8)
        .sub(core::mem::offset_of!(DOC_ITEM, entry)) as *mut DOC_ITEM;
    while &raw mut (*Item).entry != &raw mut (*infoPtr).Items {
        list_remove((&raw mut (*Item).entry).cast());
        SYSLINK_FreeDocItem(Item);
        Item = Item2;
        Item2 = ((*Item).entry.Flink as *mut u8)
            .sub(core::mem::offset_of!(DOC_ITEM, entry)) as *mut DOC_ITEM;
    }
}

unsafe extern "C" fn SYSLINK_ParseText(infoPtr: *mut SYSLINK_INFO, Text: LPCWSTR) -> UINT {
    static SL_LINKOPEN: [u16; 2] = [0x003Cu16, 0x0061];
    static SL_HREF: [u16; 6] = [0x0068u16, 0x0072, 0x0065, 0x0066, 0x003D, 0x0022];
    static SL_ID: [u16; 4] = [0x0069u16, 0x0064, 0x003D, 0x0022];
    static SL_LINKCLOSE: [u16; 4] = [0x003Cu16, 0x002F, 0x0061, 0x003E];
    let mut current: LPCWSTR = core::ptr::null();
    let mut textstart: LPCWSTR = core::ptr::null();
    let mut linktext: LPCWSTR = core::ptr::null();
    let mut firsttag: LPCWSTR = core::ptr::null();
    let mut taglen: i32 = 0;
    let mut textlen: i32 = 0;
    let mut linklen: i32 = 0;
    let mut docitems: i32 = 0;
    let mut Last: PDOC_ITEM = core::ptr::null_mut();
    let mut CurrentType: SL_ITEM_TYPE = slText;
    let mut lpID: LPCWSTR = core::ptr::null();
    let mut lpUrl: LPCWSTR = core::ptr::null();
    let mut lenId: UINT = 0;
    let mut lenUrl: UINT = 0;

    current = Text;
    while *current as i32 != 0 {
        if *current as i32 == '<' as i32 {
            if wcsnicmp(current, (SL_LINKOPEN).as_ptr(), (ARRAY_SIZE!(SL_LINKOPEN)) as _) == 0
                && (CurrentType as u32 == slText as u32)
            {
                let mut ValidParam: BOOL = FALSE;
                let mut ValidLink: BOOL = FALSE;

                if *current.offset(2 as isize) as i32 == '>' as i32 {
                    /* we just have to deal with a <a> tag */
                    taglen = 3;
                    ValidLink = TRUE;
                    ValidParam = TRUE;
                    firsttag = current;
                    linklen = 0;
                    lpID = core::ptr::null();
                    lpUrl = core::ptr::null();
                } else if *current.offset(2 as isize) as i32 == (*infoPtr).BreakChar as i32 {
                    /* we expect parameters, parse them */
                    let mut CurrentParameter: *mut LPCWSTR = core::ptr::null_mut();
                    let mut tmp: LPCWSTR = core::ptr::null();
                    let mut CurrentParameterLen: *mut UINT = core::ptr::null_mut();

                    taglen = 3;
                    tmp = current.offset(taglen as isize);
                    lpID = core::ptr::null();
                    lpUrl = core::ptr::null();

                    loop {
                        /* compare the current position with all known parameters */
                        if wcsnicmp(tmp, (SL_HREF).as_ptr(), (ARRAY_SIZE!(SL_HREF)) as _) == 0 {
                            taglen += 6;
                            ValidParam = TRUE;
                            CurrentParameter = core::ptr::addr_of_mut!(lpUrl);
                            CurrentParameterLen = core::ptr::addr_of_mut!(lenUrl);
                        } else if wcsnicmp(tmp, (SL_ID).as_ptr(), (ARRAY_SIZE!(SL_ID)) as _) == 0 {
                            taglen += 4;
                            ValidParam = TRUE;
                            CurrentParameter = core::ptr::addr_of_mut!(lpID);
                            CurrentParameterLen = core::ptr::addr_of_mut!(lenId);
                        } else {
                            ValidParam = FALSE;
                        }

                        if ValidParam != 0 {
                            /* we got a known parameter, now search until the next " character.
                               If we can't find a " character, there's a syntax error and we just assume it's text */
                            ValidParam = FALSE;
                            *CurrentParameter = current.offset(taglen as isize);
                            *CurrentParameterLen = 0;

                            tmp = *CurrentParameter;
                            while *tmp as i32 != 0 {
                                taglen += 1;
                                if *tmp as i32 == '"' as i32 {
                                    ValidParam = TRUE;
                                    tmp = tmp.offset(1);
                                    break;
                                }
                                *CurrentParameterLen = (*CurrentParameterLen).wrapping_add(1);
                                tmp = tmp.offset(1);
                            }
                        }

                        if ValidParam != 0 {
                            /* we're done with this parameter, now there are only 2 possibilities:
                             * 1. another parameter is coming, so expect a ' ' (space) character
                             * 2. the tag is being closed, so expect a '<' character
                             */
                            if *tmp as i32 == (*infoPtr).BreakChar as i32 {
                                /* we expect another parameter, do the whole thing again */
                                taglen += 1;
                                tmp = tmp.offset(1);
                                continue;
                            } else if *tmp as i32 == '>' as i32 {
                                /* the tag is being closed, we're done */
                                ValidLink = TRUE;
                                taglen += 1;
                            }
                        }
                        break;
                    }
                }

                if ValidLink != 0 && ValidParam != 0 {
                    /* the <a ...> tag appears to be valid. save all information
                       so we can add the link if we find a valid </a> tag later */
                    CurrentType = slLink;
                    linktext = current.offset(taglen as isize);
                    linklen = 0;
                    firsttag = current;
                } else {
                    taglen = 1;
                    lpID = core::ptr::null();
                    lpUrl = core::ptr::null();
                    if textstart.is_null() {
                        textstart = current;
                    }
                }
            } else if wcsnicmp(current, (SL_LINKCLOSE).as_ptr(), (ARRAY_SIZE!(SL_LINKCLOSE)) as _) == 0
                && (CurrentType as u32 == slLink as u32)
                && !firsttag.is_null()
            {
                /* there's a <a...> tag opened, first add the previous text, if present */
                if !textstart.is_null() && textlen > 0 && firsttag > textstart {
                    Last = SYSLINK_AppendDocItem(infoPtr, textstart, firsttag.offset_from(textstart) as UINT, slText, Last);
                    if Last.is_null() {
                        return docitems as UINT;
                    }
                    docitems += 1;
                    textstart = core::ptr::null();
                    textlen = 0;
                }

                /* now it's time to add the link to the document */
                current = current.offset(4 as isize);
                if !linktext.is_null() && linklen > 0 {
                    Last = SYSLINK_AppendDocItem(infoPtr, linktext, linklen as UINT, slLink, Last);
                    if Last.is_null() {
                        return docitems as UINT;
                    }
                    docitems += 1;
                    if CurrentType as u32 == slLink as u32 {
                        let mut nc: i32 = 0;

                        if ((*infoPtr).Style & (WS_DISABLED as DWORD)) == 0 {
                            (*Last).u.Link.state |= LIS_ENABLED as UINT;
                        }
                        /* Copy the tag parameters */
                        if !lpID.is_null() {
                            nc = min(lenId as i32, lstrlenW(lpID));
                            nc = min(nc, (MAX_LINKID_TEXT - 1) as i32);
                            (*Last).u.Link.szID = Alloc(
                                ((nc + 1) as usize * core::mem::size_of::<WCHAR>()) as DWORD,
                            ) as *mut WCHAR;
                            if !(*Last).u.Link.szID.is_null() {
                                lstrcpynW((*Last).u.Link.szID, lpID, nc + 1);
                            }
                        } else {
                            (*Last).u.Link.szID = core::ptr::null_mut();
                        }
                        if !lpUrl.is_null() {
                            nc = min(lenUrl as i32, lstrlenW(lpUrl));
                            nc = min(nc, (L_MAX_URL_LENGTH - 1) as i32);
                            (*Last).u.Link.szUrl = Alloc(
                                ((nc + 1) as usize * core::mem::size_of::<WCHAR>()) as DWORD,
                            ) as *mut WCHAR;
                            if !(*Last).u.Link.szUrl.is_null() {
                                lstrcpynW((*Last).u.Link.szUrl, lpUrl, nc + 1);
                            }
                        } else {
                            (*Last).u.Link.szUrl = core::ptr::null_mut();
                        }
                    }
                    linktext = core::ptr::null();
                }
                CurrentType = slText;
                firsttag = core::ptr::null();
                textstart = core::ptr::null();
                continue;
            } else {
                /* we don't know what tag it is, so just continue */
                taglen = 1;
                linklen += 1;
                if CurrentType as u32 == slText as u32 && textstart.is_null() {
                    textstart = current;
                }
            }

            textlen += taglen;
            current = current.offset(taglen as isize);
        } else {
            textlen += 1;
            linklen += 1;

            /* save the pointer of the current text item if we couldn't find a tag */
            if textstart.is_null() && CurrentType as u32 == slText as u32 {
                textstart = current;
            }

            current = current.offset(1);
        }
    }

    if !textstart.is_null() && textlen > 0 {
        Last = SYSLINK_AppendDocItem(infoPtr, textstart, textlen as UINT, CurrentType, Last);
        if Last.is_null() {
            return docitems as UINT;
        }
        if CurrentType as u32 == slLink as u32 {
            let mut nc: i32 = 0;

            if ((*infoPtr).Style & (WS_DISABLED as DWORD)) == 0 {
                (*Last).u.Link.state |= LIS_ENABLED as UINT;
            }
            /* Copy the tag parameters */
            if !lpID.is_null() {
                nc = min(lenId as i32, lstrlenW(lpID));
                nc = min(nc, (MAX_LINKID_TEXT - 1) as i32);
                (*Last).u.Link.szID = Alloc(
                    ((nc + 1) as usize * core::mem::size_of::<WCHAR>()) as DWORD,
                ) as *mut WCHAR;
                if !(*Last).u.Link.szID.is_null() {
                    lstrcpynW((*Last).u.Link.szID, lpID, nc + 1);
                }
            } else {
                (*Last).u.Link.szID = core::ptr::null_mut();
            }
            if !lpUrl.is_null() {
                nc = min(lenUrl as i32, lstrlenW(lpUrl));
                nc = min(nc, (L_MAX_URL_LENGTH - 1) as i32);
                (*Last).u.Link.szUrl = Alloc(
                    ((nc + 1) as usize * core::mem::size_of::<WCHAR>()) as DWORD,
                ) as *mut WCHAR;
                if !(*Last).u.Link.szUrl.is_null() {
                    lstrcpynW((*Last).u.Link.szUrl, lpUrl, nc + 1);
                }
            } else {
                (*Last).u.Link.szUrl = core::ptr::null_mut();
            }
        }
        docitems += 1;
    }

    if !linktext.is_null() && linklen > 0 {
        /* we got an unclosed link, just display the text */
        Last = SYSLINK_AppendDocItem(infoPtr, linktext, linklen as UINT, slText, Last);
        if Last.is_null() {
            return docitems as UINT;
        }
        docitems += 1;
    }

    return docitems as UINT;
}

unsafe extern "C" fn SYSLINK_RepaintLink(
    mut infoPtr: *const SYSLINK_INFO,
    mut DocItem: *const DOC_ITEM,
) {
    let mut bl: PDOC_TEXTBLOCK = ::core::ptr::null_mut::<DOC_TEXTBLOCK>();
    let mut n: ::core::ffi::c_int = 0;
    if (*DocItem).Type as ::core::ffi::c_uint != slLink as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/syslink.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"SYSLINK_RepaintLink\0" as *const u8 as *const ::core::ffi::c_char,
                462 as ::core::ffi::c_int,
                b"DocItem not a link!\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return;
    }
    bl = (*DocItem).Blocks;
    if !bl.is_null() {
        n = (*DocItem).nText as ::core::ffi::c_int;
        while n > 0 {
            InvalidateRect((*infoPtr).Self_, core::ptr::addr_of!((*bl).rc), TRUE);
            n -= (*bl).nChars + (*bl).nSkip;
            bl = bl.offset(1);
        }
    }
}

unsafe extern "C" fn SYSLINK_GetLinkItemByIndex(infoPtr: *const SYSLINK_INFO, mut iLink: ::core::ffi::c_int) -> PDOC_ITEM {
    let mut Current: *mut DOC_ITEM = core::ptr::null_mut();
    Current = ((*infoPtr).Items.Flink as *mut u8).offset(-(core::mem::offset_of!(DOC_ITEM, entry) as isize)) as *mut DOC_ITEM;
    while &raw mut (*Current).entry != (&raw const (*infoPtr).Items as *mut list).cast() {
        if (*Current).Type as ::core::ffi::c_uint == slLink as ::core::ffi::c_int as ::core::ffi::c_uint
            && {
                let fresh15 = iLink;
                iLink = iLink - 1;
                fresh15 <= 0
            }
        {
            return Current as PDOC_ITEM;
        }
        Current = ((*Current).entry.Flink as *mut u8).offset(-(core::mem::offset_of!(DOC_ITEM, entry) as isize)) as *mut DOC_ITEM;
    }
    return core::ptr::null_mut();
}

unsafe extern "C" fn SYSLINK_GetFocusLink(
    mut infoPtr: *const SYSLINK_INFO,
    mut LinkId: *mut ::core::ffi::c_int,
) -> PDOC_ITEM {
    let mut Current: *mut DOC_ITEM = ::core::ptr::null_mut::<DOC_ITEM>();
    let mut id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    Current = ((*infoPtr).Items.Flink as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(DOC_ITEM, entry) as isize),
    ) as *mut DOC_ITEM;
    while &raw mut (*Current).entry != (&raw const (*infoPtr).Items as *mut list).cast() {
        if (*Current).Type as ::core::ffi::c_uint
            == slLink as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if ((*Current).u.Link.state & (LIS_FOCUSED as UINT)) != 0 {
                if !LinkId.is_null() {
                    *LinkId = id;
                }
                return Current as PDOC_ITEM;
            }
            id += 1;
        }
        Current = ((*Current).entry.Flink as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(DOC_ITEM, entry) as isize),
        ) as *mut DOC_ITEM;
    }
    return ::core::ptr::null_mut::<DOC_ITEM>();
}

unsafe extern "C" fn SYSLINK_GetNextLink(
    infoPtr: *const SYSLINK_INFO,
    Current: PDOC_ITEM,
) -> PDOC_ITEM {
    let mut Next: *mut DOC_ITEM = core::ptr::null_mut::<DOC_ITEM>();
    let Head: *mut list = if !Current.is_null() {
        (&raw mut (*Current).entry).cast()
    } else {
        &raw const (*infoPtr).Items as *mut list
    };
    Next = ((*Head).next as *mut u8)
        .sub(core::mem::offset_of!(DOC_ITEM, entry)) as *mut DOC_ITEM;
    while &raw mut (*Next).entry != (Head).cast() {
        if (*Next).Type as ::core::ffi::c_uint == slLink as ::core::ffi::c_int as ::core::ffi::c_uint {
            return Next as PDOC_ITEM;
        }
        Next = ((*Next).entry.Flink as *mut u8)
            .sub(core::mem::offset_of!(DOC_ITEM, entry)) as *mut DOC_ITEM;
    }
    core::ptr::null_mut::<DOC_ITEM>() as PDOC_ITEM
}

unsafe extern "C" fn SYSLINK_GetPrevLink(
    infoPtr: *const SYSLINK_INFO,
    Current: PDOC_ITEM,
) -> PDOC_ITEM {
    let mut Prev: *mut DOC_ITEM = core::ptr::null_mut();
    let Head: *mut _LIST_ENTRY = if !Current.is_null() {
        core::ptr::addr_of_mut!((*Current).entry)
    } else {
        list_tail((core::ptr::addr_of!((*infoPtr).Items)).cast()) as *mut _LIST_ENTRY
    };
    let Off: usize = core::mem::offset_of!(DOC_ITEM, entry);
    Prev = ((*Head).Blink as *mut u8).offset(-(Off as isize)) as *mut DOC_ITEM;
    while core::ptr::addr_of_mut!((*Prev).entry) != Head {
        if (*Prev).Type as u32 == slLink as u32 {
            return Prev as PDOC_ITEM;
        }
        Prev = ((*Prev).entry.Blink as *mut u8).offset(-(Off as isize)) as *mut DOC_ITEM;
    }
    return core::ptr::null_mut();
}

unsafe extern "C" fn SYSLINK_WrapLine(
    mut Text: LPWSTR,
    mut BreakChar: WCHAR,
    mut x: ::core::ffi::c_int,
    mut LineLen: *mut ::core::ffi::c_int,
    mut nFit: ::core::ffi::c_int,
    mut Extent: LPSIZE,
) -> BOOL {
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < nFit {
        if *Text.offset(i as isize) as ::core::ffi::c_int == '\r' as i32
            || *Text.offset(i as isize) as ::core::ffi::c_int == '\n' as i32
        {
            break;
        }
        i += 1;
    }
    if i == *LineLen {
        return FALSE;
    }
    if *Text.offset(i as isize) as ::core::ffi::c_int != '\r' as i32
        && *Text.offset(i as isize) as ::core::ffi::c_int != '\n' as i32
        && *Text.offset(i as isize) as ::core::ffi::c_int != BreakChar as ::core::ffi::c_int
    {
        while i != 0
            && *Text.offset((i - 1) as isize) as ::core::ffi::c_int
                != BreakChar as ::core::ffi::c_int
        {
            i -= 1;
        }
        if i == 0 as ::core::ffi::c_int {
            (*Extent).cx = 0 as LONG;
            (*Extent).cy = 0 as LONG;
            if x == SL_LEFTMARGIN {
                i = max(nFit, 1);
            }
        }
    }
    *LineLen = i;
    return TRUE;
}

unsafe extern "C" fn SYSLINK_Render(infoPtr: *const SYSLINK_INFO, hdc: HDC, pRect: PRECT) {
    let mut rc: RECT = core::mem::zeroed();
    let mut Current: PDOC_ITEM = core::ptr::null_mut();
    let mut hOldFont: HGDIOBJ = core::ptr::null_mut();
    let mut x: i32;
    let mut y: i32;
    let mut LineHeight: i32;
    let mut szDoc: SIZE = core::mem::zeroed();
    let mut tm: TEXTMETRICW = core::mem::zeroed();

    szDoc.cy = 0 as LONG;
    szDoc.cx = szDoc.cy;

    rc = *pRect;
    rc.right -= SL_RIGHTMARGIN as LONG;
    rc.bottom -= SL_BOTTOMMARGIN as LONG;

    if (rc.right as i32 - SL_LEFTMARGIN as i32) < (0 as i32) {
        rc.right = MAXLONG as LONG;
    }
    if (rc.bottom as i32 - SL_TOPMARGIN as i32) < (0 as i32) {
        rc.bottom = MAXLONG as LONG;
    }

    hOldFont = SelectObject(hdc, (*infoPtr).Font as HGDIOBJ);

    x = SL_LEFTMARGIN as i32;
    y = SL_TOPMARGIN as i32;
    GetTextMetricsW(hdc, core::ptr::addr_of_mut!(tm));
    LineHeight = (tm.tmHeight + tm.tmExternalLeading) as i32;

    Current = ((*infoPtr).Items.Flink as *mut u8).offset(
        -(core::mem::offset_of!(DOC_ITEM, entry) as isize),
    ) as *mut _DOC_ITEM as PDOC_ITEM;
    while core::ptr::addr_of_mut!((*Current).entry)
        != (core::ptr::addr_of!((*infoPtr).Items) as *mut list).cast()
    {
        let mut n: i32 = 0;
        let mut nBlocks: i32 = 0;
        let mut tx: LPWSTR = core::ptr::null_mut();
        let mut bl: PDOC_TEXTBLOCK = core::ptr::null_mut();
        let mut cbl: PDOC_TEXTBLOCK = core::ptr::null_mut();
        let mut nFit: INT = 0;
        let mut szDim: SIZE = core::mem::zeroed();
        let mut SkipChars: i32 = 0 as i32;

        if !((*Current).nText == 0 as UINT) {
            tx = core::ptr::addr_of_mut!((*Current).Text) as *mut WCHAR as LPWSTR;
            n = (*Current).nText as i32;
            Free((*Current).Blocks as LPVOID);
            (*Current).Blocks = core::ptr::null_mut();
            bl = core::ptr::null_mut();
            nBlocks = 0 as i32;

            if (*Current).Type as u32 == slText as u32 {
                SelectObject(hdc, (*infoPtr).Font as HGDIOBJ);
            } else if (*Current).Type as u32 == slLink as u32 {
                SelectObject(hdc, (*infoPtr).LinkFont as HGDIOBJ);
            }

            while n > 0 as i32 {
                /* skip break characters unless they're the first of the doc item */
                if tx != core::ptr::addr_of_mut!((*Current).Text) as *mut WCHAR
                    || x == SL_LEFTMARGIN as i32
                {
                    if n != 0 && *tx as i32 == '\r' as i32 {
                        tx = tx.offset(1);
                        SkipChars += 1;
                        n -= 1;
                    }
                    if n != 0 && *tx as i32 == '\n' as i32 {
                        tx = tx.offset(1);
                        SkipChars += 1;
                        n -= 1;
                    }
                    while n > 0 as i32
                        && *tx as i32 == (*infoPtr).BreakChar as i32
                    {
                        tx = tx.offset(1);
                        SkipChars += 1;
                        n -= 1;
                    }
                }

                if (n == 0 as i32 && SkipChars != 0 as i32)
                    || GetTextExtentExPointW(
                        hdc,
                        tx as LPCWSTR,
                        n,
                        rc.right as i32 - x,
                        core::ptr::addr_of_mut!(nFit),
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(szDim),
                    ) != 0
                {
                    let mut LineLen: i32 = n;
                    let mut Wrap: BOOL = FALSE as BOOL;
                    let mut nbl: PDOC_TEXTBLOCK = core::ptr::null_mut();

                    if n != 0 as i32 {
                        Wrap = SYSLINK_WrapLine(
                            tx,
                            (*infoPtr).BreakChar,
                            x,
                            core::ptr::addr_of_mut!(LineLen),
                            nFit as i32,
                            core::ptr::addr_of_mut!(szDim),
                        );

                        if LineLen == 0 as i32 {
                            /* move one line down, the word didn't fit into the line */
                            x = SL_LEFTMARGIN as i32;
                            y += LineHeight;
                            continue;
                        } else if LineLen != n {
                            if GetTextExtentExPointW(
                                hdc,
                                tx as LPCWSTR,
                                LineLen,
                                rc.right as i32 - x,
                                core::ptr::null_mut(),
                                core::ptr::null_mut(),
                                core::ptr::addr_of_mut!(szDim),
                            ) == 0
                            {
                                if !bl.is_null() {
                                    Free(bl as LPVOID);
                                    bl = core::ptr::null_mut();
                                    nBlocks = 0 as i32;
                                }
                                break;
                            }
                        }
                    }

                    nbl = ReAlloc(
                        bl as LPVOID,
                        ((nBlocks + 1 as i32) as usize)
                            .wrapping_mul(core::mem::size_of::<DOC_TEXTBLOCK>() as usize)
                            as DWORD,
                    ) as PDOC_TEXTBLOCK;
                    if !nbl.is_null() {
                        bl = nbl;
                        nBlocks += 1;

                        cbl = bl
                            .offset(nBlocks as isize)
                            .offset(-(1 as isize));

                        (*cbl).nChars = LineLen;
                        (*cbl).nSkip = SkipChars;
                        SetRect(
                            core::ptr::addr_of_mut!((*cbl).rc),
                            x,
                            y,
                            x + szDim.cx as i32,
                            y + szDim.cy as i32,
                        );

                        if (*cbl).rc.right > szDoc.cx {
                            szDoc.cx = (*cbl).rc.right;
                        }
                        if (*cbl).rc.bottom > szDoc.cy {
                            szDoc.cy = (*cbl).rc.bottom;
                        }

                        if LineLen != 0 as i32 {
                            x += szDim.cx as i32;
                            if Wrap != 0 {
                                x = SL_LEFTMARGIN as i32;
                                y += LineHeight;
                            }
                        }
                    } else {
                        Free(bl as LPVOID);
                        bl = core::ptr::null_mut();
                        nBlocks = 0 as i32;

                        break;
                    }
                    n -= LineLen;
                    tx = tx.offset(LineLen as isize);
                    SkipChars = 0 as i32;
                } else {
                    n -= 1;
                }
            }

            if nBlocks != 0 as i32 {
                (*Current).Blocks = bl;
            }
        }

        Current = ((*Current).entry.Flink as *mut u8).offset(
            -(core::mem::offset_of!(DOC_ITEM, entry) as isize),
        ) as *mut _DOC_ITEM as PDOC_ITEM;
    }

    SelectObject(hdc, hOldFont);

    (*pRect).right = (*pRect).left + szDoc.cx;
    (*pRect).bottom = (*pRect).top + szDoc.cy;
}

unsafe extern "C" fn SYSLINK_Draw(infoPtr: *const SYSLINK_INFO, hdc: HDC) -> LRESULT {
    let mut rc: RECT = core::mem::zeroed();
    let mut Current: PDOC_ITEM = core::ptr::null_mut();
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut OldTextColor: COLORREF = 0;
    let mut OldBkColor: COLORREF = 0;
    let mut hBrush: HBRUSH = core::ptr::null_mut();
    let mut text_flags: UINT = ETO_CLIPPED;
    let mut mode: UINT = GetBkMode(hdc) as UINT;

    hOldFont = SelectObject(hdc, (*infoPtr).Font as HGDIOBJ) as HFONT;
    OldTextColor = SetTextColor(hdc, (*infoPtr).TextColor);
    OldBkColor = SetBkColor(hdc, comctl32_color.clrWindow);

    GetClientRect((*infoPtr).Self_, core::ptr::addr_of_mut!(rc));
    rc.right -= SL_RIGHTMARGIN + SL_LEFTMARGIN;
    rc.bottom -= SL_BOTTOMMARGIN + SL_TOPMARGIN;

    if rc.right < 0 || rc.bottom < 0 {
        return 0;
    }

    hBrush = SendMessageW(
        (*infoPtr).Notify,
        WM_CTLCOLORSTATIC,
        hdc as WPARAM,
        (*infoPtr).Self_ as LPARAM,
    ) as HBRUSH;
    if ((*infoPtr).Style & LWS_TRANSPARENT) == 0 {
        FillRect(hdc, core::ptr::addr_of_mut!(rc), hBrush);
        if GetBkMode(hdc) == (OPAQUE as i32) {
            text_flags |= ETO_OPAQUE;
        }
    } else {
        SetBkMode(hdc, (TRANSPARENT as i32));
    }

    // syslink.c: `#ifndef __REACTOS__`. The brush is the parent's WM_CTLCOLORSTATIC answer,
    // which the parent owns; ReactOS does not delete it.
    if cfg!(not_reactos) {
        DeleteObject((hBrush).cast());
    }

    // LIST_FOR_EACH_ENTRY(Current, &infoPtr->Items, DOC_ITEM, entry)
    Current = ((*infoPtr).Items.Flink as *mut u8)
        .sub(core::mem::offset_of!(DOC_ITEM, entry)) as PDOC_ITEM;
    while core::ptr::addr_of_mut!((*Current).entry)
        != (core::ptr::addr_of!((*infoPtr).Items) as *mut list).cast()
    {
        let mut n: ::core::ffi::c_int = 0;
        let mut tx: LPWSTR = core::ptr::null_mut();
        let mut bl: PDOC_TEXTBLOCK = core::ptr::null_mut();

        bl = (*Current).Blocks;
        if !bl.is_null() {
            tx = core::ptr::addr_of_mut!((*Current).Text) as *mut WCHAR as LPWSTR;
            n = (*Current).nText as ::core::ffi::c_int;

            if (*Current).Type as ::core::ffi::c_uint == slText as ::core::ffi::c_uint {
                SelectObject(hdc, (*infoPtr).Font as HGDIOBJ);
                SetTextColor(hdc, (*infoPtr).TextColor);
            } else {
                SelectObject(hdc, (*infoPtr).LinkFont as HGDIOBJ);
                SetTextColor(
                    hdc,
                    if ((*Current).u.Link.state & LIS_VISITED) == 0 {
                        (*infoPtr).LinkColor
                    } else {
                        (*infoPtr).VisitedColor
                    },
                );
            }

            while n > 0 {
                tx = tx.offset((*bl).nSkip as isize);
                ExtTextOutW(
                    hdc,
                    (*bl).rc.left as ::core::ffi::c_int,
                    (*bl).rc.top as ::core::ffi::c_int,
                    text_flags,
                    core::ptr::addr_of_mut!((*bl).rc),
                    tx as LPCWSTR,
                    (*bl).nChars as UINT,
                    core::ptr::null::<INT>(),
                );
                if (*Current).Type as ::core::ffi::c_uint == slLink as ::core::ffi::c_uint
                    && ((*Current).u.Link.state & LIS_FOCUSED) != 0
                    && (*infoPtr).HasFocus != 0
                {
                    let mut PrevTextColor: COLORREF = 0;
                    PrevTextColor = SetTextColor(hdc, (*infoPtr).TextColor);
                    DrawFocusRect(hdc, core::ptr::addr_of_mut!((*bl).rc) as LPCRECT);
                    SetTextColor(hdc, PrevTextColor);
                }
                tx = tx.offset((*bl).nChars as isize);
                n -= (*bl).nChars + (*bl).nSkip;
                bl = bl.offset(1);
            }
        }

        Current = ((*Current).entry.Flink as *mut u8)
            .sub(core::mem::offset_of!(DOC_ITEM, entry)) as PDOC_ITEM;
    }

    SetBkColor(hdc, OldBkColor);
    SetTextColor(hdc, OldTextColor);
    SelectObject(hdc, hOldFont as HGDIOBJ);
    SetBkMode(hdc, mode as ::core::ffi::c_int);
    return 0;
}

unsafe extern "C" fn SYSLINK_Paint(infoPtr: *const SYSLINK_INFO, hdcParam: HDC) -> LRESULT {
    let mut hdc: HDC;
    let mut ps: PAINTSTRUCT = core::mem::zeroed();

    hdc = if !hdcParam.is_null() { hdcParam } else { BeginPaint((*infoPtr).Self_, core::ptr::addr_of_mut!(ps)) };
    if !hdc.is_null() {
        SYSLINK_Draw(infoPtr, hdc);
        if hdcParam.is_null() {
            EndPaint((*infoPtr).Self_, core::ptr::addr_of_mut!(ps));
        }
    }
    return 0;
}

unsafe extern "C" fn SYSLINK_SetFont(infoPtr: *mut SYSLINK_INFO, hFont: HFONT, bRedraw: BOOL) -> HFONT {
    let mut hdc: HDC = core::ptr::null_mut();
    let mut lf: LOGFONTW = core::mem::zeroed();
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    let mut rcClient: RECT = core::mem::zeroed();
    let mut hOldFont: HFONT = (*infoPtr).Font;
    (*infoPtr).Font = hFont;

    /* free the underline font */
    if !(*infoPtr).LinkFont.is_null() {
        DeleteObject((*infoPtr).LinkFont as HGDIOBJ);
        (*infoPtr).LinkFont = core::ptr::null_mut();
    }

    /* Render text position and word wrapping in memory */
    if GetClientRect((*infoPtr).Self_, &raw mut rcClient) != 0 {
        hdc = GetDC((*infoPtr).Self_);
        if !hdc.is_null() {
            /* create a new underline font */
            if GetTextMetricsW(hdc, &raw mut tm) != 0
                && GetObjectW(
                    (*infoPtr).Font as HANDLE,
                    core::mem::size_of::<LOGFONTW>() as ::core::ffi::c_int,
                    &raw mut lf as LPVOID,
                ) != 0
            {
                lf.lfUnderline = TRUE as BYTE;
                (*infoPtr).LinkFont = CreateFontIndirectW(&raw mut lf);
                (*infoPtr).BreakChar = tm.tmBreakChar;
            } else {
            }

            SYSLINK_Render(infoPtr, hdc, &raw mut rcClient);
            ReleaseDC((*infoPtr).Self_, hdc);
        }
    }

    if bRedraw != 0 {
        RedrawWindow(
            (*infoPtr).Self_,
            core::ptr::null::<RECT>(),
            core::ptr::null_mut::<HRGN__>(),
            (RDW_INVALIDATE | RDW_UPDATENOW) as UINT,
        );
    }

    return hOldFont;
}

unsafe extern "C" fn SYSLINK_SetText(infoPtr: *mut SYSLINK_INFO, Text: LPCWSTR) -> LRESULT {
    SYSLINK_ClearDoc(infoPtr);

    if Text.is_null() || *Text == 0 {
        return (TRUE) as LRESULT;
    }

    /* let's parse the string and create a document */
    if SYSLINK_ParseText(infoPtr, Text) > 0 {
        let mut rcClient: RECT = core::mem::zeroed();

        /* Render text position and word wrapping in memory */
        if GetClientRect((*infoPtr).Self_, &mut rcClient as *mut RECT) != 0 {
            let hdc: HDC = GetDC((*infoPtr).Self_);
            if !hdc.is_null() {
                SYSLINK_Render(infoPtr, hdc, &mut rcClient as *mut RECT);
                ReleaseDC((*infoPtr).Self_, hdc);

                InvalidateRect((*infoPtr).Self_, core::ptr::null::<RECT>(), TRUE);
            }
        }
    }

    return (TRUE) as LRESULT;
}

unsafe extern "C" fn SYSLINK_SetFocusLink(
    infoPtr: *const SYSLINK_INFO,
    DocItem: *const DOC_ITEM,
) -> PDOC_ITEM {
    let mut Current: PDOC_ITEM = core::ptr::null_mut::<DOC_ITEM>();
    let mut PrevFocus: PDOC_ITEM = core::ptr::null_mut::<DOC_ITEM>();
    Current = ((*infoPtr).Items.Flink as *mut core::ffi::c_char).offset(
        -(core::mem::offset_of!(DOC_ITEM, entry) as isize),
    ) as *mut DOC_ITEM as PDOC_ITEM;
    while core::ptr::addr_of_mut!((*Current).entry)
        != (core::ptr::addr_of!((*infoPtr).Items) as *mut list).cast()
    {
        if (*Current).Type as core::ffi::c_uint == slLink as core::ffi::c_int as core::ffi::c_uint {
            if PrevFocus.is_null() && ((*Current).u.Link.state & (LIS_FOCUSED as UINT)) != 0 {
                PrevFocus = Current;
            }
            if Current == DocItem as PDOC_ITEM {
                (*Current).u.Link.state |= LIS_FOCUSED as UINT;
            } else {
                (*Current).u.Link.state &= !(LIS_FOCUSED as UINT);
            }
        }
        Current = ((*Current).entry.Flink as *mut core::ffi::c_char).offset(
            -(core::mem::offset_of!(DOC_ITEM, entry) as isize),
        ) as *mut DOC_ITEM as PDOC_ITEM;
    }
    return PrevFocus;
}

unsafe extern "C" fn SYSLINK_SetItem(infoPtr: *const SYSLINK_INFO, Item: *const LITEM) -> LRESULT {
    let mut di: PDOC_ITEM = ::core::ptr::null_mut::<_DOC_ITEM>();
    let mut nc: ::core::ffi::c_int = 0;
    let mut szId: PWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut szUrl: PWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut Repaint: BOOL = FALSE;

    if (*Item).mask & (LIF_ITEMINDEX) as UINT == 0
        || (*Item).mask & (LIF_FLAGSMASK) as UINT == 0
    {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/syslink.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"SYSLINK_SetItem\0" as *const u8 as *const ::core::ffi::c_char,
                1007 as ::core::ffi::c_int,
                b"Invalid Flags!\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return (FALSE) as LRESULT;
    }

    di = SYSLINK_GetLinkItemByIndex(infoPtr, (*Item).iLink);
    if di.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/syslink.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"SYSLINK_SetItem\0" as *const u8 as *const ::core::ffi::c_char,
                1014 as ::core::ffi::c_int,
                b"Link %d couldn't be found\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                (*Item).iLink,
            );
        }
        return (FALSE) as LRESULT;
    }

    if (*Item).mask & (LIF_ITEMID) as UINT != 0 {
        nc = min(
            lstrlenW(&raw const (*Item).szID as LPCWSTR),
            ((MAX_LINKID_TEXT) - (1)) as i32,
        );
        szId = Alloc(
            ((nc + (1)) as usize)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
        ) as PWSTR;
        if !szId.is_null() {
            lstrcpynW(
                szId as LPWSTR,
                &raw const (*Item).szID as LPCWSTR,
                nc + (1),
            );
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/syslink.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"SYSLINK_SetItem\0" as *const u8 as *const ::core::ffi::c_char,
                    1028 as ::core::ffi::c_int,
                    b"Unable to allocate memory for link id\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            return (FALSE) as LRESULT;
        }
    }

    if (*Item).mask & (LIF_URL) as UINT != 0 {
        nc = min(
            lstrlenW(&raw const (*Item).szUrl as LPCWSTR),
            ((L_MAX_URL_LENGTH) - (1)) as i32,
        );
        szUrl = Alloc(
            ((nc + (1)) as usize)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
        ) as PWSTR;
        if !szUrl.is_null() {
            lstrcpynW(
                szUrl as LPWSTR,
                &raw const (*Item).szUrl as LPCWSTR,
                nc + (1),
            );
        } else {
            Free(szId as LPVOID);
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_2,
                    __dbch_2,
                    b"dll/win32/comctl32/syslink.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"SYSLINK_SetItem\0" as *const u8 as *const ::core::ffi::c_char,
                    1045 as ::core::ffi::c_int,
                    b"Unable to allocate memory for link url\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            return (FALSE) as LRESULT;
        }
    }

    if (*Item).mask & (LIF_ITEMID) as UINT != 0 {
        Free((*di).u.Link.szID as LPVOID);
        (*di).u.Link.szID = szId as *mut WCHAR;
    }

    if (*Item).mask & (LIF_URL) as UINT != 0 {
        Free((*di).u.Link.szUrl as LPVOID);
        (*di).u.Link.szUrl = szUrl as *mut WCHAR;
    }

    if (*Item).mask & (LIF_STATE) as UINT != 0 {
        let mut oldstate: UINT = (*di).u.Link.state;
        (*di).u.Link.state &= !((*Item).stateMask & (LIS_MASK) as UINT);
        (*di).u.Link.state |=
            (*Item).state & (*Item).stateMask & (LIS_MASK) as UINT;
        Repaint = (oldstate != (*di).u.Link.state) as ::core::ffi::c_int as BOOL;
        SYSLINK_SetFocusLink(
            infoPtr,
            (if (*di).u.Link.state & (LIS_FOCUSED) as UINT != 0 {
                di
            } else {
                ::core::ptr::null_mut::<_DOC_ITEM>()
            }) as *const DOC_ITEM,
        );
    }

    if Repaint != 0 {
        SYSLINK_RepaintLink(infoPtr, di as *const DOC_ITEM);
    }

    return (TRUE) as LRESULT;
}

unsafe extern "C" fn SYSLINK_GetItem(
    mut infoPtr: *const SYSLINK_INFO,
    mut Item: PLITEM,
) -> LRESULT {
    let mut di: PDOC_ITEM = ::core::ptr::null_mut::<_DOC_ITEM>();
    if (*Item).mask & (LIF_ITEMINDEX as UINT) == 0
        || (*Item).mask & (LIF_FLAGSMASK as UINT) == 0
    {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/syslink.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"SYSLINK_GetItem\0" as *const u8 as *const ::core::ffi::c_char,
                1093 as ::core::ffi::c_int,
                b"Invalid Flags!\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return FALSE as LRESULT;
    }
    di = SYSLINK_GetLinkItemByIndex(infoPtr, (*Item).iLink);
    if di.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/syslink.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"SYSLINK_GetItem\0" as *const u8 as *const ::core::ffi::c_char,
                1100 as ::core::ffi::c_int,
                b"Link %d couldn't be found\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                (*Item).iLink,
            );
        }
        return FALSE as LRESULT;
    }
    if (*Item).mask & (LIF_STATE as UINT) != 0 {
        (*Item).state = (*di).u.Link.state & (*Item).stateMask;
        if (*infoPtr).HasFocus == 0 {
            (*Item).state &= !(LIS_FOCUSED as UINT);
        }
    }
    if (*Item).mask & (LIF_ITEMID as UINT) != 0 {
        if !(*di).u.Link.szID.is_null() {
            lstrcpyW(
                &raw mut (*Item).szID as LPWSTR,
                (*di).u.Link.szID as LPCWSTR,
            );
        } else {
            (*Item).szID[0 as usize] = 0 as WCHAR;
        }
    }
    if (*Item).mask & (LIF_URL as UINT) != 0 {
        if !(*di).u.Link.szUrl.is_null() {
            lstrcpyW(
                &raw mut (*Item).szUrl as LPWSTR,
                (*di).u.Link.szUrl as LPCWSTR,
            );
        } else {
            (*Item).szUrl[0 as usize] = 0 as WCHAR;
        }
    }
    return TRUE as LRESULT;
}

unsafe extern "C" fn SYSLINK_PtInDocItem(DocItem: *const DOC_ITEM, pt: POINT) -> BOOL {
    let mut bl: PDOC_TEXTBLOCK = core::ptr::null_mut();
    let mut n: i32 = 0;
    bl = (*DocItem).Blocks;
    if !bl.is_null() {
        n = (*DocItem).nText as i32;
        while n > 0 {
            if PtInRect(core::ptr::addr_of_mut!((*bl).rc), pt) != 0 {
                return TRUE;
            }
            n -= (*bl).nChars + (*bl).nSkip;
            bl = bl.offset(1);
        }
    }
    return FALSE;
}

unsafe extern "C" fn SYSLINK_HitTest(infoPtr: *const SYSLINK_INFO, HitTest: PLHITTESTINFO) -> LRESULT {
    let mut Current: PDOC_ITEM = core::ptr::null_mut::<DOC_ITEM>();
    let mut id: core::ffi::c_int = 0 as core::ffi::c_int;
    Current = ((*infoPtr).Items.Flink as *mut u8)
        .sub(core::mem::offset_of!(DOC_ITEM, entry)) as *mut DOC_ITEM as PDOC_ITEM;
    while core::ptr::addr_of_mut!((*Current).entry)
        != core::ptr::addr_of!((*infoPtr).Items) as *mut _
    {
        if (*Current).Type as core::ffi::c_uint == slLink as core::ffi::c_int as core::ffi::c_uint {
            if SYSLINK_PtInDocItem(Current as *const DOC_ITEM, (*HitTest).pt) != 0 {
                (*HitTest).item.mask = 0 as UINT;
                (*HitTest).item.iLink = id;
                (*HitTest).item.state = 0 as UINT;
                (*HitTest).item.stateMask = 0 as UINT;
                if !(*Current).u.Link.szID.is_null() {
                    lstrcpyW(
                        core::ptr::addr_of_mut!((*HitTest).item.szID) as LPWSTR,
                        (*Current).u.Link.szID as LPCWSTR,
                    );
                } else {
                    (*HitTest).item.szID[0 as usize] = 0 as WCHAR;
                }
                if !(*Current).u.Link.szUrl.is_null() {
                    lstrcpyW(
                        core::ptr::addr_of_mut!((*HitTest).item.szUrl) as LPWSTR,
                        (*Current).u.Link.szUrl as LPCWSTR,
                    );
                } else {
                    (*HitTest).item.szUrl[0 as usize] = 0 as WCHAR;
                }
                return TRUE as LRESULT;
            }
            id += 1;
        }
        Current = ((*Current).entry.Flink as *mut u8)
            .sub(core::mem::offset_of!(DOC_ITEM, entry)) as *mut DOC_ITEM as PDOC_ITEM;
    }
    return FALSE as LRESULT;
}

unsafe extern "C" fn SYSLINK_GetIdealHeight(infoPtr: *const SYSLINK_INFO) -> LRESULT {
    let mut hdc: HDC = GetDC((*infoPtr).Self_);
    if !hdc.is_null() {
        let mut height: LRESULT = 0;
        let mut tm: TEXTMETRICW = core::mem::zeroed();
        let mut hOldFont: HGDIOBJ = SelectObject(hdc, (*infoPtr).Font as HGDIOBJ);
        if GetTextMetricsW(hdc, core::ptr::addr_of_mut!(tm)) != 0 {
            height = tm.tmHeight as LRESULT;
        } else {
            height = 0;
        }
        SelectObject(hdc, hOldFont);
        ReleaseDC((*infoPtr).Self_, hdc);
        return height;
    }
    return 0;
}

unsafe extern "C" fn SYSLINK_SendParentNotify(infoPtr: *const SYSLINK_INFO, code: UINT, Link: *const DOC_ITEM, iLink: ::core::ffi::c_int) -> LRESULT {
    let mut nml: NMLINK = tagNMLINK {
        hdr: tagNMHDR {
            hwndFrom: ::core::ptr::null_mut::<HWND__>(),
            idFrom: 0,
            code: 0,
        },
        item: tagLITEM {
            mask: 0,
            iLink: 0,
            state: 0,
            stateMask: 0,
            szID: [0; 48],
            szUrl: [0; 2084],
        },
    };
    nml.hdr.hwndFrom = (*infoPtr).Self_;
    nml.hdr.idFrom = GetWindowLongPtrW((*infoPtr).Self_, GWLP_ID) as UINT_PTR;
    nml.hdr.code = code;
    nml.item.mask = 0 as UINT;
    nml.item.iLink = iLink;
    nml.item.state = 0 as UINT;
    nml.item.stateMask = 0 as UINT;
    if !(*Link).u.Link.szID.is_null() {
        lstrcpyW(
            &raw mut nml.item.szID as LPWSTR,
            (*Link).u.Link.szID as LPCWSTR,
        );
    } else {
        nml.item.szID[0 as usize] = 0 as WCHAR;
    }
    if !(*Link).u.Link.szUrl.is_null() {
        lstrcpyW(
            &raw mut nml.item.szUrl as LPWSTR,
            (*Link).u.Link.szUrl as LPCWSTR,
        );
    } else {
        nml.item.szUrl[0 as usize] = 0 as WCHAR;
    }
    return SendMessageW(
        (*infoPtr).Notify,
        WM_NOTIFY as UINT,
        nml.hdr.idFrom as WPARAM,
        &raw mut nml as LPARAM,
    );
}

unsafe extern "C" fn SYSLINK_SetFocus(infoPtr: *mut SYSLINK_INFO) -> LRESULT {
    let mut Focus: PDOC_ITEM = core::ptr::null_mut();

    (*infoPtr).HasFocus = TRUE;

    /* We always select the first link, even if we activated the control using
       SHIFT+TAB. This is the default behavior */
    Focus = SYSLINK_GetNextLink(infoPtr, core::ptr::null_mut());
    if !Focus.is_null() {
        SYSLINK_SetFocusLink(infoPtr, Focus as *const DOC_ITEM);
        SYSLINK_RepaintLink(infoPtr, Focus as *const DOC_ITEM);
    }
    return 0;
}

unsafe extern "C" fn SYSLINK_KillFocus(mut infoPtr: *mut SYSLINK_INFO) -> LRESULT {
    let mut Focus: PDOC_ITEM = core::ptr::null_mut::<DOC_ITEM>();
    (*infoPtr).HasFocus = FALSE as BOOL;
    Focus = SYSLINK_GetFocusLink(infoPtr, core::ptr::null_mut::<core::ffi::c_int>());
    if !Focus.is_null() {
        SYSLINK_RepaintLink(infoPtr, Focus as *const DOC_ITEM);
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn SYSLINK_LinkAtPt(infoPtr: *const SYSLINK_INFO, pt: *const POINT, LinkId: *mut c_int, MustBeEnabled: BOOL) -> PDOC_ITEM {
    let mut Current: PDOC_ITEM;
    let mut id: c_int = 0;

    let ListHead: *mut list = core::ptr::addr_of!((*infoPtr).Items) as *mut list;
    Current = ((*infoPtr).Items.Flink as *mut c_char)
        .offset(-(core::mem::offset_of!(DOC_ITEM, entry) as isize)) as *mut DOC_ITEM as PDOC_ITEM;
    while core::ptr::addr_of_mut!((*Current).entry) != (ListHead).cast() {
        if ((*Current).Type as c_uint == slLink as c_int as c_uint)
            && (SYSLINK_PtInDocItem(Current as *const DOC_ITEM, *pt) != 0)
            && (MustBeEnabled == 0 || ((*Current).u.Link.state & (LIS_ENABLED as UINT)) != 0)
        {
            if !LinkId.is_null() {
                *LinkId = id;
            }
            return Current;
        }
        id += 1;
        Current = ((*Current).entry.Flink as *mut c_char)
            .offset(-(core::mem::offset_of!(DOC_ITEM, entry) as isize)) as *mut DOC_ITEM as PDOC_ITEM;
    }

    return core::ptr::null_mut();
}

unsafe extern "C" fn SYSLINK_LButtonDown(infoPtr: *mut SYSLINK_INFO, pt: *const POINT) -> LRESULT {
    let mut Current: PDOC_ITEM;
    let mut Old: PDOC_ITEM;
    let mut id: core::ffi::c_int = 0;

    Current = SYSLINK_LinkAtPt(infoPtr, pt, core::ptr::addr_of_mut!(id), TRUE);
    if !Current.is_null() {
        SetFocus((*infoPtr).Self_);

        Old = SYSLINK_SetFocusLink(infoPtr, Current as *const DOC_ITEM);
        if !Old.is_null() && Old != Current {
            SYSLINK_RepaintLink(infoPtr, Old as *const DOC_ITEM);
        }
        (*infoPtr).MouseDownID = id;
        SYSLINK_RepaintLink(infoPtr, Current as *const DOC_ITEM);
    }

    return 0;
}

unsafe extern "C" fn SYSLINK_LButtonUp(infoPtr: *mut SYSLINK_INFO, pt: *const POINT) -> LRESULT {
    if (*infoPtr).MouseDownID > -(1 as ::core::ffi::c_int) {
        let mut Current: PDOC_ITEM = core::ptr::null_mut();
        let mut id: ::core::ffi::c_int = 0;
        Current = SYSLINK_LinkAtPt(infoPtr, pt, core::ptr::addr_of_mut!(id), TRUE);
        if !Current.is_null()
            && ((*Current).u.Link.state & (LIS_FOCUSED as UINT)) != 0
            && (*infoPtr).MouseDownID == id
        {
            SYSLINK_SendParentNotify(infoPtr, NM_CLICK, Current as *const DOC_ITEM, id);
        }
    }

    (*infoPtr).MouseDownID = -(1 as ::core::ffi::c_int);

    return 0 as LRESULT;
}

unsafe extern "C" fn SYSLINK_OnEnter(infoPtr: *const SYSLINK_INFO) -> BOOL {
    if (*infoPtr).HasFocus != 0 && (*infoPtr).IgnoreReturn == 0 {
        let mut Focus: PDOC_ITEM = core::ptr::null_mut();
        let mut id: i32 = 0;
        Focus = SYSLINK_GetFocusLink(infoPtr, core::ptr::addr_of_mut!(id));
        if !Focus.is_null() {
            SYSLINK_SendParentNotify(infoPtr, NM_RETURN, Focus, id);
            return TRUE;
        }
    }
    return FALSE;
}

unsafe extern "C" fn SYSKEY_SelectNextPrevLink(infoPtr: *const SYSLINK_INFO, Prev: BOOL) -> BOOL {
    if (*infoPtr).HasFocus != 0 {
        let mut Focus: PDOC_ITEM = core::ptr::null_mut();
        let mut id: core::ffi::c_int = 0;
        Focus = SYSLINK_GetFocusLink(infoPtr, core::ptr::addr_of_mut!(id));
        if !Focus.is_null() {
            let mut NewFocus: PDOC_ITEM = core::ptr::null_mut();
            let mut OldFocus: PDOC_ITEM = core::ptr::null_mut();
            if Prev != 0 {
                NewFocus = SYSLINK_GetPrevLink(infoPtr, Focus);
            } else {
                NewFocus = SYSLINK_GetNextLink(infoPtr, Focus);
            }
            if !NewFocus.is_null() {
                OldFocus = SYSLINK_SetFocusLink(infoPtr, NewFocus as *const DOC_ITEM);
                if !OldFocus.is_null() && OldFocus != NewFocus {
                    SYSLINK_RepaintLink(infoPtr, OldFocus as *const DOC_ITEM);
                }
                SYSLINK_RepaintLink(infoPtr, NewFocus as *const DOC_ITEM);
                return TRUE;
            }
        }
    }
    return FALSE;
}

unsafe extern "C" fn SYSLINK_NoNextLink(infoPtr: *const SYSLINK_INFO, Prev: BOOL) -> BOOL {
    let mut Focus: PDOC_ITEM;
    let mut NewFocus: PDOC_ITEM;
    Focus = SYSLINK_GetFocusLink(infoPtr, core::ptr::null_mut());
    if Prev != 0 {
        NewFocus = SYSLINK_GetPrevLink(infoPtr, Focus);
    } else {
        NewFocus = SYSLINK_GetNextLink(infoPtr, Focus);
    }

    return (NewFocus == core::ptr::null_mut()) as BOOL;
}

unsafe extern "C" fn SYSLINK_GetIdealSize(infoPtr: *const SYSLINK_INFO, cxMaxWidth: i32, lpSize: *mut SIZE) -> LONG {
    let mut rc: RECT = core::mem::zeroed();
    let mut hdc: HDC;

    rc.left = 0 as LONG;
    rc.top = rc.left;
    rc.bottom = rc.top;
    rc.right = cxMaxWidth as LONG;

    hdc = GetDC((*infoPtr).Self_);
    if !hdc.is_null() {
        let hOldFont: HGDIOBJ = SelectObject(hdc, (*infoPtr).Font as HGDIOBJ);

        SYSLINK_Render(infoPtr, hdc, &raw mut rc);

        SelectObject(hdc, hOldFont);
        ReleaseDC((*infoPtr).Self_, hdc);

        (*lpSize).cx = rc.right;
        (*lpSize).cy = rc.bottom;
    }

    return rc.bottom;
}
unsafe extern "system" fn SysLinkWindowProc(hwnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut infoPtr: *mut SYSLINK_INFO = core::ptr::null_mut();

    infoPtr = GetWindowLongPtrW(hwnd, 0) as *mut SYSLINK_INFO;

    if infoPtr.is_null() && message != WM_CREATE {
        return DefWindowProcW(hwnd, message, wParam, lParam);
    }

    match message {
        WM_PRINTCLIENT | WM_PAINT => {
            return SYSLINK_Paint(infoPtr, wParam as HDC);
        }

        WM_ERASEBKGND => {
            if ((*infoPtr).Style & LWS_TRANSPARENT as DWORD) == 0 {
                let mut hdc: HDC = wParam as HDC;
                let mut brush: HBRUSH = CreateSolidBrush(comctl32_color.clrWindow);
                let mut rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };

                GetClipBox(hdc, core::ptr::addr_of_mut!(rect));
                FillRect(hdc, core::ptr::addr_of_mut!(rect), brush);
                DeleteObject(brush as HGDIOBJ);
                return 1 as LRESULT;
            }
            return 0 as LRESULT;
        }

        WM_SETCURSOR => {
            let mut ht: LHITTESTINFO = core::mem::zeroed();
            let mut mp: DWORD = GetMessagePos();

            ht.pt.x = LOWORD!(mp) as i16 as LONG;
            ht.pt.y = HIWORD!(mp) as i16 as LONG;

            ScreenToClient((*infoPtr).Self_, core::ptr::addr_of_mut!(ht.pt));
            if SYSLINK_HitTest(infoPtr, core::ptr::addr_of_mut!(ht)) != 0 {
                SetCursor(LoadCursorW(
                    core::ptr::null_mut(),
                    IDC_HAND as LPCWSTR,
                ));
                return TRUE as LRESULT;
            }

            return DefWindowProcW(hwnd, message, wParam, lParam);
        }

        WM_SIZE => {
            let mut rcClient: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            if GetClientRect((*infoPtr).Self_, core::ptr::addr_of_mut!(rcClient)) != 0 {
                let mut hdc: HDC = GetDC((*infoPtr).Self_);
                if !hdc.is_null() {
                    SYSLINK_Render(infoPtr, hdc, core::ptr::addr_of_mut!(rcClient));
                    ReleaseDC((*infoPtr).Self_, hdc);
                }
            }
            return 0 as LRESULT;
        }

        WM_GETFONT => {
            return (*infoPtr).Font as LRESULT;
        }

        WM_SETFONT => {
            return SYSLINK_SetFont(infoPtr, wParam as HFONT, lParam as BOOL) as LRESULT;
        }

        WM_SETTEXT => {
            SYSLINK_SetText(infoPtr, lParam as LPWSTR);
            return DefWindowProcW(hwnd, message, wParam, lParam);
        }

        WM_LBUTTONDOWN => {
            let mut pt: POINT = POINT { x: 0, y: 0 };
            pt.x = LOWORD!(lParam as DWORD) as i16 as LONG;
            pt.y = HIWORD!(lParam as DWORD) as i16 as LONG;
            return SYSLINK_LButtonDown(infoPtr, core::ptr::addr_of_mut!(pt));
        }

        WM_LBUTTONUP => {
            let mut pt: POINT = POINT { x: 0, y: 0 };
            pt.x = LOWORD!(lParam as DWORD) as i16 as LONG;
            pt.y = HIWORD!(lParam as DWORD) as i16 as LONG;
            return SYSLINK_LButtonUp(infoPtr, core::ptr::addr_of_mut!(pt));
        }

        WM_KEYDOWN => {
            match wParam as u32 {
                VK_RETURN => {
                    SYSLINK_OnEnter(infoPtr);
                    return 0 as LRESULT;
                }
                VK_TAB => {
                    let mut shift: BOOL = (GetKeyState((VK_SHIFT) as i32) as BOOL) & 0x8000;
                    SYSKEY_SelectNextPrevLink(infoPtr, shift);
                    return 0 as LRESULT;
                }
                _ => {
                    return DefWindowProcW(hwnd, message, wParam, lParam);
                }
            }
        }

        WM_GETDLGCODE => {
            let mut Ret: LRESULT = DLGC_HASSETSEL as LRESULT;
            let mut vk: ::core::ffi::c_int = if lParam != 0 as LPARAM {
                (*(lParam as LPMSG)).wParam as ::core::ffi::c_int
            } else {
                (0) as ::core::ffi::c_int
            };
            match (vk) as u32 {
                VK_RETURN => {
                    Ret |= DLGC_WANTMESSAGE as LRESULT;
                }
                VK_TAB => {
                    let mut shift: BOOL = (GetKeyState((VK_SHIFT) as i32) as BOOL) & 0x8000;
                    if SYSLINK_NoNextLink(infoPtr, shift) == 0 {
                        Ret |= DLGC_WANTTAB as LRESULT;
                    } else {
                        Ret |= DLGC_WANTCHARS as LRESULT;
                    }
                }
                _ => {}
            }
            return Ret;
        }

        WM_NCHITTEST => {
            let mut pt: POINT = POINT { x: 0, y: 0 };
            let mut rc: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            pt.x = LOWORD!(lParam as DWORD) as i16 as LONG;
            pt.y = HIWORD!(lParam as DWORD) as i16 as LONG;

            GetClientRect((*infoPtr).Self_, core::ptr::addr_of_mut!(rc));
            ScreenToClient((*infoPtr).Self_, core::ptr::addr_of_mut!(pt));
            if pt.x < 0 as LONG
                || pt.y < 0 as LONG
                || pt.x > rc.right
                || pt.y > rc.bottom
            {
                return HTNOWHERE as LRESULT;
            }

            if !SYSLINK_LinkAtPt(
                infoPtr,
                core::ptr::addr_of_mut!(pt),
                core::ptr::null_mut::<::core::ffi::c_int>(),
                FALSE,
            )
            .is_null()
            {
                return HTCLIENT as LRESULT;
            }

            return HTTRANSPARENT as LRESULT;
        }

        LM_HITTEST => {
            return SYSLINK_HitTest(infoPtr, lParam as PLHITTESTINFO);
        }

        LM_SETITEM => {
            return SYSLINK_SetItem(infoPtr, lParam as PLITEM as *const LITEM);
        }

        LM_GETITEM => {
            return SYSLINK_GetItem(infoPtr, lParam as PLITEM);
        }

        LM_GETIDEALHEIGHT => {
            if lParam != 0 {
                return (SYSLINK_GetIdealSize(infoPtr, wParam as ::core::ffi::c_int, lParam as *mut SIZE)) as LRESULT;
            } else {
                return SYSLINK_GetIdealHeight(infoPtr);
            }
        }

        WM_SETFOCUS => {
            return SYSLINK_SetFocus(infoPtr);
        }

        WM_KILLFOCUS => {
            return SYSLINK_KillFocus(infoPtr);
        }

        WM_ENABLE => {
            (*infoPtr).Style &= !(WS_DISABLED as DWORD);
            (*infoPtr).Style |= (if wParam != 0 {
                0
            } else {
                WS_DISABLED
            }) as DWORD;
            InvalidateRect((*infoPtr).Self_, core::ptr::null::<RECT>(), FALSE);
            return 0 as LRESULT;
        }

        WM_STYLECHANGED => {
            if wParam == GWL_STYLE as WPARAM {
                (*infoPtr).Style = (*(lParam as LPSTYLESTRUCT)).styleNew;

                InvalidateRect((*infoPtr).Self_, core::ptr::null::<RECT>(), TRUE);
            }
            return 0 as LRESULT;
        }

        WM_CREATE => {
            let mut cs: *mut CREATESTRUCTW = lParam as *mut CREATESTRUCTW;

            /* allocate memory for info struct */
            infoPtr = Alloc(core::mem::size_of::<SYSLINK_INFO>() as DWORD) as *mut SYSLINK_INFO;
            if infoPtr.is_null() {
                return -(1 as LRESULT);
            }
            SetWindowLongPtrW(hwnd, 0, infoPtr as DWORD_PTR as LONG_PTR);

            /* initialize the info struct */
            (*infoPtr).Self_ = hwnd;
            (*infoPtr).Notify = (*cs).hwndParent;
            (*infoPtr).Style = (*cs).style as DWORD;
            (*infoPtr).Font = 0 as HFONT;
            (*infoPtr).LinkFont = 0 as HFONT;
            list_init((core::ptr::addr_of_mut!((*infoPtr).Items)).cast());
            (*infoPtr).HasFocus = FALSE as BOOL;
            (*infoPtr).MouseDownID = -(1 as ::core::ffi::c_int);
            (*infoPtr).TextColor = comctl32_color.clrWindowText;
            (*infoPtr).LinkColor = comctl32_color.clrHighlight;
            (*infoPtr).VisitedColor = comctl32_color.clrHighlight;
            (*infoPtr).BreakChar = ' ' as i32 as WCHAR;
            (*infoPtr).IgnoreReturn = ((*infoPtr).Style & LWS_IGNORERETURN as DWORD) as BOOL;

            SYSLINK_SetText(infoPtr, (*cs).lpszName);
            return 0 as LRESULT;
        }

        WM_DESTROY => {
            SYSLINK_ClearDoc(infoPtr);
            if false {
                /* ifndef __REACTOS__ */
                if (*infoPtr).Font != 0 as HFONT {
                    DeleteObject((*infoPtr).Font as HGDIOBJ);
                }
            }
            if (*infoPtr).LinkFont != 0 as HFONT {
                DeleteObject((*infoPtr).LinkFont as HGDIOBJ);
            }
            SetWindowLongPtrW(hwnd, 0, 0 as LONG_PTR);
            Free(infoPtr as LPVOID);
            return 0 as LRESULT;
        }

        WM_SYSCOLORCHANGE => {
            COMCTL32_RefreshSysColors();
            return 0 as LRESULT;
        }

        _ => {
            if (message >= (WM_USER as UINT))
                && (message < (WM_APP as UINT))
                && COMCTL32_IsReflectedMessage(message) == 0
            {
            }
            return DefWindowProcW(hwnd, message, wParam, lParam);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn SYSLINK_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    ZeroMemory!(
        &raw mut wndClass as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSW>(),
    );
    wndClass.style = (CS_GLOBALCLASS | CS_VREDRAW | CS_HREDRAW) as UINT;
    wndClass.lpfnWndProc = Some(SysLinkWindowProc);
    wndClass.cbClsExtra = 0 as core::ffi::c_int;
    wndClass.cbWndExtra = core::mem::size_of::<*mut SYSLINK_INFO>() as core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        IDC_ARROW as LPWSTR as LPCWSTR,
    );
    wndClass.lpszClassName = (WC_LINK).as_ptr();

    RegisterClassW(&raw mut wndClass);
}
#[no_mangle]
pub unsafe extern "C" fn SYSLINK_Unregister() {
    UnregisterClassW((WC_LINK).as_ptr() as LPCWSTR, core::ptr::null_mut());
}
