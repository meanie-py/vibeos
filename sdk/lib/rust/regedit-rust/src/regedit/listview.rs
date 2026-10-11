use crate::regedit::treeview::GetItemPath;
use crate::regedit::edit::RenameValue;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_010340BC5B61: [u16; 11] = [
    0x0052u16, 0x0045, 0x0047, 0x005F, 0x0042, 0x0049, 0x004E, 0x0041, 0x0052, 0x0059,
    0x0000,
];
pub static RUSTCONV_WSTR_138BA0474808: [u16; 6] = [0x0025u16, 0x0030, 0x0032, 0x0058, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_22559A0A7652: [u16; 13] = [
    0x0052u16, 0x0045, 0x0047, 0x005F, 0x004D, 0x0055, 0x004C, 0x0054, 0x0049, 0x005F,
    0x0053, 0x005A, 0x0000,
];
pub static RUSTCONV_WSTR_30807FFEA6F9: [u16; 31] = [
    0x0052u16, 0x0045, 0x0047, 0x005F, 0x0052, 0x0045, 0x0053, 0x004F, 0x0055, 0x0052,
    0x0043, 0x0045, 0x005F, 0x0052, 0x0045, 0x0051, 0x0055, 0x0049, 0x0052, 0x0045,
    0x004D, 0x0045, 0x004E, 0x0054, 0x0053, 0x005F, 0x004C, 0x0049, 0x0053, 0x0054,
    0x0000,
];
pub static RUSTCONV_WSTR_30BC907EFB89: [u16; 7] = [0x0052u16, 0x0045, 0x0047, 0x005F, 0x0053, 0x005A, 0x0000];
pub static RUSTCONV_WSTR_3B3A24BC138B: [u16; 10] = [0x0052u16, 0x0045, 0x0047, 0x005F, 0x0044, 0x0057, 0x004F, 0x0052, 0x0044, 0x0000];
pub static RUSTCONV_WSTR_4D71FACE03C3: [u16; 14] = [
    0x0052u16, 0x0045, 0x0047, 0x005F, 0x0045, 0x0058, 0x0050, 0x0041, 0x004E, 0x0044,
    0x005F, 0x0053, 0x005A, 0x0000,
];
pub static RUSTCONV_WSTR_75C0322241F2: [u16; 10] = [0x0052u16, 0x0045, 0x0047, 0x005F, 0x0051, 0x0057, 0x004F, 0x0052, 0x0044, 0x0000];
pub static RUSTCONV_WSTR_8D71B3FAAB82: [u16; 2] = [0x0020u16, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_ACE6EAA358C6: [u16; 9] = [0x0052u16, 0x0045, 0x0047, 0x005F, 0x004C, 0x0049, 0x004E, 0x004B, 0x0000];
pub static RUSTCONV_WSTR_CE539A27F3CF: [u16; 10] = [0x004Cu16, 0x0069, 0x0073, 0x0074, 0x0020, 0x0056, 0x0069, 0x0065, 0x0077, 0x0000];
pub static RUSTCONV_WSTR_D8DCD81B58B7: [u16; 21] = [
    0x0052u16, 0x0045, 0x0047, 0x005F, 0x0044, 0x0057, 0x004F, 0x0052, 0x0044, 0x005F,
    0x0042, 0x0049, 0x0047, 0x005F, 0x0045, 0x004E, 0x0044, 0x0049, 0x0041, 0x004E,
    0x0000,
];
pub static RUSTCONV_WSTR_DD7670DDD9D8: [u16; 18] = [
    0x0052u16, 0x0045, 0x0047, 0x005F, 0x0052, 0x0045, 0x0053, 0x004F, 0x0055, 0x0052,
    0x0043, 0x0045, 0x005F, 0x004C, 0x0049, 0x0053, 0x0054, 0x0000,
];
pub static RUSTCONV_WSTR_F01ADE64740B: [u16; 12] = [
    0x0030u16, 0x0078, 0x0025, 0x0030, 0x0038, 0x0078, 0x0020, 0x0028, 0x0025, 0x0075,
    0x0029, 0x0000,
];
pub static RUSTCONV_WSTR_F13F2352AA73: [u16; 29] = [
    0x0052u16, 0x0045, 0x0047, 0x005F, 0x0046, 0x0055, 0x004C, 0x004C, 0x005F, 0x0052,
    0x0045, 0x0053, 0x004F, 0x0055, 0x0052, 0x0043, 0x0045, 0x005F, 0x0044, 0x0045,
    0x0053, 0x0043, 0x0052, 0x0049, 0x0050, 0x0054, 0x004F, 0x0052, 0x0000,
];
pub static RUSTCONV_WSTR_F43A50ADA987: [u16; 9] = [0x0052u16, 0x0045, 0x0047, 0x005F, 0x004E, 0x004F, 0x004E, 0x0045, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut Image_String: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut Image_Bin: ::core::ffi::c_int = 0;

#[no_mangle]
pub static mut iListViewSelect: INT = -(1 as INT);

static mut g_iSortedColumn: INT = 0;

static mut default_column_widths: [::core::ffi::c_int; 3] = [
    35 as ::core::ffi::c_int,
    25 as ::core::ffi::c_int,
    40 as ::core::ffi::c_int,
];

static mut column_alignment: [::core::ffi::c_int; 3] = [(LVCFMT_LEFT as i32), (LVCFMT_LEFT as i32), (LVCFMT_LEFT as i32)];

pub const CX_ICON: i32 = 16;

pub const CY_ICON: i32 = 16;

pub const LISTVIEW_NUM_ICONS: i32 = 2;

#[repr(C)]
pub struct tagLINE_INFO {
    pub dwValType: DWORD,
    pub name: LPWSTR,
    pub val: *mut core::ffi::c_void,
    pub val_len: usize,
}
pub type LINE_INFO = tagLINE_INFO;
pub type PLINE_INFO = *mut tagLINE_INFO;

#[repr(C)]
pub struct tagSORT_INFO {
    pub iSortingColumn: INT,
    pub bSortAscending: BOOL,
}
pub type SORT_INFO = tagSORT_INFO;
pub type PSORT_INFO = *mut tagSORT_INFO;

// MAX_LIST_COLUMNS = (IDS_LIST_COLUMN_LAST - IDS_LIST_COLUMN_FIRST + 1)
// Resolved from the 3-element initializers below.
pub const MAX_LIST_COLUMNS: usize = 3;

// LVCFMT_LEFT (commctrl.h)
#[no_mangle]
pub unsafe extern "C" fn GetValueName(mut hwndLV: HWND, mut iStartAt: ::core::ffi::c_int) -> *mut WCHAR {
    let mut item: core::ffi::c_int;
    let mut LVItem: LVITEMW = core::mem::zeroed();
    let mut lineinfo: PLINE_INFO;

    item = ListView_GetNextItem!(hwndLV, iStartAt, LVNI_SELECTED);
    if item == -1 {
        return core::ptr::null_mut();
    }

    LVItem.iItem = item;
    LVItem.iSubItem = 0;
    LVItem.mask = LVIF_PARAM;
    if (ListView_GetItem!(hwndLV, core::ptr::addr_of_mut!(LVItem))) as u32 == FALSE {
        return core::ptr::null_mut();
    }

    lineinfo = LVItem.lParam as PLINE_INFO;
    if lineinfo.is_null() {
        return core::ptr::null_mut();
    }

    return (*lineinfo).name as *mut WCHAR;
}
#[no_mangle]
pub unsafe extern "C" fn SetValueName(hwndLV: HWND, pszValueName: LPCWSTR) {
    let mut i: INT;
    let mut c: INT;
    let mut fi: LVFINDINFOW = core::mem::zeroed();

    c = ListView_GetItemCount!(hwndLV);
    i = 0;
    while i < c {
        ListView_SetItemState!(hwndLV, i, 0, LVIS_FOCUSED | LVIS_SELECTED);
        i += 1;
    }
    if pszValueName.is_null()
        || *pszValueName.offset(0 as isize) == 0
    {
        i = 0;
    } else {
        fi.flags = LVFI_STRING;
        fi.psz = pszValueName;
        i = ListView_FindItem!(hwndLV, -(1), core::ptr::addr_of_mut!(fi));
    }
    ListView_SetItemState!(
        hwndLV,
        i,
        LVIS_FOCUSED | LVIS_SELECTED,
        LVIS_FOCUSED | LVIS_SELECTED,
    );
    ListView_EnsureVisible!(hwndLV, i, FALSE);
    iListViewSelect = i;
}
#[no_mangle]
pub unsafe extern "C" fn IsDefaultValue(hwndLV: HWND, i: ::core::ffi::c_int) -> BOOL {
    let mut lineinfo: PLINE_INFO;
    let mut Item: LVITEMW = core::mem::zeroed();

    Item.mask = LVIF_PARAM;
    Item.iItem = i;
    if ListView_GetItem!(hwndLV, core::ptr::addr_of_mut!(Item)) != 0 {
        lineinfo = Item.lParam as PLINE_INFO;
        return (!lineinfo.is_null()
            && ((*lineinfo).name.is_null()
                || wcscmp((*lineinfo).name as *const u16, RUSTCONV_WSTR_96A296D224F2.as_ptr()) == 0))
            as BOOL;
    }
    return (FALSE) as BOOL;
}

unsafe extern "C" fn AddEntryToList(hwndLV: HWND, Name: LPWSTR, dwValType: DWORD, ValBuf: *mut core::ffi::c_void, dwCount: DWORD, Position: i32, ValExists: BOOL) {
    let mut linfo: PLINE_INFO = core::ptr::null_mut();
    let mut item: LVITEMW = core::mem::zeroed();
    let mut index: i32;

    linfo = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<LINE_INFO>() + dwCount as usize) as SIZE_T) as PLINE_INFO;
    (*linfo).dwValType = dwValType;
    (*linfo).val_len = dwCount as size_t;
    if dwCount > 0 {
        memcpy(linfo.add(1 as usize) as *mut core::ffi::c_void, ValBuf, dwCount as size_t);
        (*linfo).val = linfo.add(1 as usize) as *mut core::ffi::c_void;
    }
    (*linfo).name = _wcsdup(Name as *const u16) as LPWSTR;

    item.mask = (LVIF_TEXT | LVIF_PARAM | LVIF_IMAGE) as UINT;
    item.iItem = if Position == -1 { 0 } else { Position };
    item.iSubItem = 0;
    item.state = 0;
    item.stateMask = 0;
    item.pszText = Name;
    item.cchTextMax = wcslen(item.pszText as *const u16) as i32;
    if item.cchTextMax == 0 {
        item.pszText = LPSTR_TEXTCALLBACK;
    }
    item.iImage = 0;
    item.lParam = linfo as LPARAM;
    match dwValType {
        REG_SZ | REG_EXPAND_SZ | REG_MULTI_SZ => {
            item.iImage = Image_String;
        }
        _ => {
            item.iImage = Image_Bin;
        }
    }

    /*    item.lParam = (LPARAM)ValBuf; */
    item.iIndent = 0;

    index = ListView_InsertItem!(hwndLV, core::ptr::addr_of_mut!(item));
    if index != -1 {
        match dwValType {
            REG_SZ | REG_EXPAND_SZ => {
                if dwCount > 0 {
                    ListView_SetItemText!(hwndLV, index, 2, ValBuf);
                } else if ValExists == 0 {
                    let mut buffer: [WCHAR; 255] = [0; 255];
                    /* load (value not set) string */
                    LoadStringW(hInst, IDS_VALUE_NOT_SET, core::ptr::addr_of_mut!(buffer) as *mut WCHAR, ARRAY_SIZE!(buffer) as i32);
                    ListView_SetItemText!(hwndLV, index, 2, core::ptr::addr_of_mut!(buffer) as *mut WCHAR);
                }
            }
            REG_MULTI_SZ => {
                let mut src: LPWSTR;
                let mut str: LPWSTR;
                if dwCount >= 2 {
                    src = ValBuf as LPWSTR;
                    str = HeapAlloc(GetProcessHeap(), 0, (dwCount as usize + core::mem::size_of::<WCHAR>()) as SIZE_T) as LPWSTR;
                    if !str.is_null() {
                        *str = 0x0000u16;
                        /* concatenate all srings */
                        while *src != 0x0000u16 {
                            wcscat(str as *mut u16, src as *const u16);
                            wcscat(str as *mut u16, RUSTCONV_WSTR_8D71B3FAAB82.as_ptr());
                            src = src.add((wcslen(src as *const u16) + 1) as usize);
                        }
                        ListView_SetItemText!(hwndLV, index, 2, str);
                        HeapFree(GetProcessHeap(), 0, str as *mut core::ffi::c_void);
                    } else {
                        ListView_SetItemText!(hwndLV, index, 2, RUSTCONV_WSTR_96A296D224F2.as_ptr());
                    }
                } else {
                    ListView_SetItemText!(hwndLV, index, 2, RUSTCONV_WSTR_96A296D224F2.as_ptr());
                }
            }
            REG_DWORD => {
                let mut buf: [WCHAR; 200] = [0; 200];
                if dwCount == core::mem::size_of::<DWORD>() as DWORD {
                    wsprintf(core::ptr::addr_of_mut!(buf) as *mut WCHAR, RUSTCONV_WSTR_F01ADE64740B.as_ptr(), *(ValBuf as *mut DWORD), *(ValBuf as *mut DWORD));
                } else {
                    LoadStringW(hInst, IDS_INVALID_DWORD, core::ptr::addr_of_mut!(buf) as *mut WCHAR, ARRAY_SIZE!(buf) as i32);
                }
                ListView_SetItemText!(hwndLV, index, 2, core::ptr::addr_of_mut!(buf) as *mut WCHAR);
            }
            _ => {
                let mut i: u32;
                let mut pData: LPBYTE = ValBuf as LPBYTE;
                let mut strBinary: LPWSTR;
                if dwCount > 0 {
                    strBinary = HeapAlloc(GetProcessHeap(), 0, (((dwCount as usize) * core::mem::size_of::<WCHAR>() * (3 as usize)) + core::mem::size_of::<WCHAR>()) as SIZE_T) as LPWSTR;
                    i = 0;
                    while i < dwCount {
                        wsprintf(strBinary.add((i * 3) as usize), RUSTCONV_WSTR_138BA0474808.as_ptr(), *pData.add(i as usize) as i32);
                        i += 1;
                    }
                    *strBinary.add((dwCount * 3) as usize) = 0;
                    ListView_SetItemText!(hwndLV, index, 2, strBinary);
                    HeapFree(GetProcessHeap(), 0, strBinary as *mut core::ffi::c_void);
                } else {
                    let mut szText: [WCHAR; 128] = [0; 128];
                    LoadStringW(hInst, IDS_BINARY_EMPTY, core::ptr::addr_of_mut!(szText) as *mut WCHAR, ARRAY_SIZE!(szText) as i32);
                    ListView_SetItemText!(hwndLV, index, 2, core::ptr::addr_of_mut!(szText) as *mut WCHAR);
                }
            }
        }
    }
}

unsafe extern "C" fn CreateListColumns(hWndListView: HWND, cxTotal: INT) -> BOOL {
    let mut szText: [WCHAR; 50] = [0; 50];
    let mut index: INT;
    let mut lvC: LVCOLUMNW = core::mem::zeroed();

    /* Create columns. */
    lvC.mask = (LVCF_FMT | LVCF_WIDTH | LVCF_TEXT | LVCF_SUBITEM) as UINT;
    lvC.pszText = szText.as_mut_ptr();

    /* Load the column labels from the resource file. */
    index = 0;
    while index < (MAX_LIST_COLUMNS as i32) {
        lvC.iSubItem = index;
        lvC.cx = (cxTotal * default_column_widths[index as usize]) / 100;
        lvC.fmt = column_alignment[index as usize];
        LoadStringW(hInst, (IDS_LIST_COLUMN_FIRST + (index) as u32) as UINT, szText.as_mut_ptr(), ARRAY_SIZE!(szText) as INT);
        if ListView_InsertColumn!(hWndListView, index, core::ptr::addr_of_mut!(lvC)) == -1 {
            return (FALSE) as BOOL;
        }
        index += 1;
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn InitListViewImageLists(hwndLV: HWND) -> BOOL {
    let mut himl: HIMAGELIST = core::ptr::null_mut();
    let mut hico: HICON = core::ptr::null_mut();

    himl = ImageList_Create(CX_ICON, CY_ICON,
                                 ILC_MASK, 0, LISTVIEW_NUM_ICONS);
    if himl.is_null() {
        return (FALSE) as BOOL;
    }

    hico = LoadIconW(hInst, MAKEINTRESOURCEW!(IDI_BIN));
    Image_Bin = ImageList_AddIcon!(himl, hico);

    hico = LoadIconW(hInst, MAKEINTRESOURCEW!(IDI_STRING));
    Image_String = ImageList_AddIcon!(himl, hico);

    if ImageList_GetImageCount(himl) < LISTVIEW_NUM_ICONS {
        return (FALSE) as BOOL;
    }

    let _ = ListView_SetImageList!(hwndLV, himl, LVSIL_SMALL);

    return (TRUE) as BOOL;
}

unsafe extern "C" fn OnGetDispInfo(plvdi: *mut NMLVDISPINFOW) {
    static mut buffer: [WCHAR; 200] = [0; 200];

    (*plvdi).item.pszText = core::ptr::null_mut();
    (*plvdi).item.cchTextMax = 0;

    match (*plvdi).item.iSubItem {
        0 => {
            LoadStringW(
                hInst,
                IDS_DEFAULT_VALUE_NAME,
                core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPWSTR,
                ARRAY_SIZE!(buffer) as i32,
            );
            (*plvdi).item.pszText = core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPWSTR;
        }
        1 => {
            match (*((*plvdi).item.lParam as *mut LINE_INFO)).dwValType {
                REG_NONE => {
                    (*plvdi).item.pszText = RUSTCONV_WSTR_F43A50ADA987.as_ptr() as LPWSTR;
                }
                REG_SZ => {
                    (*plvdi).item.pszText = RUSTCONV_WSTR_30BC907EFB89.as_ptr() as LPWSTR;
                }
                REG_EXPAND_SZ => {
                    (*plvdi).item.pszText = RUSTCONV_WSTR_4D71FACE03C3.as_ptr() as LPWSTR;
                }
                REG_BINARY => {
                    (*plvdi).item.pszText = RUSTCONV_WSTR_010340BC5B61.as_ptr() as LPWSTR;
                }
                REG_DWORD => {
                    (*plvdi).item.pszText = RUSTCONV_WSTR_3B3A24BC138B.as_ptr() as LPWSTR;
                }
                REG_DWORD_BIG_ENDIAN => {
                    (*plvdi).item.pszText = RUSTCONV_WSTR_D8DCD81B58B7.as_ptr() as LPWSTR;
                }
                REG_LINK => {
                    (*plvdi).item.pszText = RUSTCONV_WSTR_ACE6EAA358C6.as_ptr() as LPWSTR;
                }
                REG_MULTI_SZ => {
                    (*plvdi).item.pszText = RUSTCONV_WSTR_22559A0A7652.as_ptr() as LPWSTR;
                }
                REG_RESOURCE_LIST => {
                    (*plvdi).item.pszText = RUSTCONV_WSTR_DD7670DDD9D8.as_ptr() as LPWSTR;
                }
                REG_FULL_RESOURCE_DESCRIPTOR => {
                    (*plvdi).item.pszText = RUSTCONV_WSTR_F13F2352AA73.as_ptr() as LPWSTR;
                }
                REG_RESOURCE_REQUIREMENTS_LIST => {
                    (*plvdi).item.pszText = RUSTCONV_WSTR_30807FFEA6F9.as_ptr() as LPWSTR;
                }
                REG_QWORD => {
                    (*plvdi).item.pszText = RUSTCONV_WSTR_75C0322241F2.as_ptr() as LPWSTR;
                }
                _ => {
                    let mut buf2: [WCHAR; 200] = [0; 200];
                    LoadStringW(
                        hInst,
                        IDS_UNKNOWN_TYPE,
                        core::ptr::addr_of_mut!(buf2) as *mut WCHAR as LPWSTR,
                        ARRAY_SIZE!(buf2) as i32,
                    );
                    wsprintf(
                        core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPWSTR,
                        core::ptr::addr_of_mut!(buf2) as *mut WCHAR as LPCWSTR,
                        (*((*plvdi).item.lParam as *mut LINE_INFO)).dwValType,
                    );
                    (*plvdi).item.pszText = core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPWSTR;
                }
            }
        }
        3 => {
            (*plvdi).item.pszText = RUSTCONV_WSTR_96A296D224F2.as_ptr() as LPWSTR;
        }
        _ => {}
    }
}

unsafe extern "system" fn CompareFunc(
    lParam1: LPARAM,
    lParam2: LPARAM,
    lParamSort: LPARAM,
) -> ::core::ffi::c_int {
    let pSortInfo: PSORT_INFO = lParamSort as PSORT_INFO;
    let mut l: *mut LINE_INFO = ::core::ptr::null_mut::<LINE_INFO>();
    let mut r: *mut LINE_INFO = ::core::ptr::null_mut::<LINE_INFO>();
    let mut dw1: DWORD = 0;
    let mut dw2: DWORD = 0;
    let mut qw1: DWORDLONG = 0;
    let mut qw2: DWORDLONG = 0;

    l = lParam1 as *mut LINE_INFO;
    r = lParam2 as *mut LINE_INFO;

    if (*pSortInfo).iSortingColumn == 1 && (*l).dwValType != (*r).dwValType {
        /* Sort by type */
        if (*pSortInfo).bSortAscending != 0 {
            return (*l).dwValType as ::core::ffi::c_int - (*r).dwValType as ::core::ffi::c_int;
        } else {
            return (*r).dwValType as ::core::ffi::c_int - (*l).dwValType as ::core::ffi::c_int;
        }
    }
    if (*pSortInfo).iSortingColumn == 2 {
        /* Sort by value */
        if (*l).dwValType != (*r).dwValType {
            if (*pSortInfo).bSortAscending != 0 {
                return (*l).dwValType as ::core::ffi::c_int - (*r).dwValType as ::core::ffi::c_int;
            } else {
                return (*r).dwValType as ::core::ffi::c_int - (*l).dwValType as ::core::ffi::c_int;
            }
        }

        if (*l).val.is_null() && (*r).val.is_null() {
            return 0;
        }

        if (*pSortInfo).bSortAscending != 0 {
            if (*l).val.is_null() {
                return -1;
            }
            if (*r).val.is_null() {
                return 1;
            }
        } else {
            if (*l).val.is_null() {
                return 1;
            }
            if (*r).val.is_null() {
                return -1;
            }
        }

        let sw: DWORD = (*l).dwValType;
        if sw == REG_DWORD {
            /* REG_DWORD */
            dw1 = *((*l).val as *mut DWORD);
            dw2 = *((*r).val as *mut DWORD);
            if (*pSortInfo).bSortAscending != 0 {
                return dw1 as ::core::ffi::c_int - dw2 as ::core::ffi::c_int;
            } else {
                return dw2 as ::core::ffi::c_int - dw1 as ::core::ffi::c_int;
            }
        } else if sw == REG_QWORD {
            /* REG_QWORD_LITTLE_ENDIAN */
            qw1 = *((*l).val as *mut DWORDLONG);
            qw2 = *((*r).val as *mut DWORDLONG);
            if (*pSortInfo).bSortAscending != 0 {
                return qw1 as ::core::ffi::c_int - qw2 as ::core::ffi::c_int;
            } else {
                return qw2 as ::core::ffi::c_int - qw1 as ::core::ffi::c_int;
            }
        } else if sw == REG_SZ
            || sw == REG_EXPAND_SZ
            || sw == REG_MULTI_SZ
        {
            /* REG_SZ, REG_EXPAND_SZ, REG_MULTI_SZ */
            let mut nCompare: INT;
            let nCommonLength: size_t =
                min!((*l).val_len, (*r).val_len) / ::core::mem::size_of::<WCHAR>() as size_t;
            if (*pSortInfo).bSortAscending != 0 {
                nCompare = StrCmpNW(
                    (*l).val as *mut WCHAR as LPCWSTR,
                    (*r).val as *mut WCHAR as LPCWSTR,
                    nCommonLength as INT,
                ) as INT;
                if nCompare == 0 {
                    nCompare = (*l).val_len.wrapping_sub((*r).val_len) as INT;
                }
            } else {
                nCompare = StrCmpNW(
                    (*r).val as *mut WCHAR as LPCWSTR,
                    (*l).val as *mut WCHAR as LPCWSTR,
                    nCommonLength as INT,
                ) as INT;
                if nCompare == 0 {
                    nCompare = (*r).val_len.wrapping_sub((*l).val_len) as INT;
                }
            }
            return nCompare as ::core::ffi::c_int;
        } else {
            /* default */
            let mut nCompare: INT = 0;

            if (*pSortInfo).bSortAscending != 0 {
                nCompare = memcmp(
                    (*l).val,
                    (*r).val,
                    min!((*l).val_len, (*r).val_len),
                ) as INT;
                if nCompare == 0 {
                    nCompare = (*l).val_len.wrapping_sub((*r).val_len) as INT;
                }
            } else {
                nCompare = memcmp(
                    (*r).val,
                    (*l).val,
                    min!((*r).val_len, (*l).val_len),
                ) as INT;
                if nCompare == 0 {
                    nCompare = (*r).val_len.wrapping_sub((*l).val_len) as INT;
                }
            }

            return nCompare as ::core::ffi::c_int;
        }
    }

    /* Sort by name */
    return if (*pSortInfo).bSortAscending != 0 {
        StrCmpLogicalW((*l).name as LPCWSTR, (*r).name as LPCWSTR)
    } else {
        StrCmpLogicalW((*r).name as LPCWSTR, (*l).name as LPCWSTR)
    };
}

unsafe extern "C" fn ListView_Sort(hListView: HWND, iSortingColumn: INT, iSortedColumn: INT) -> BOOL {
    if (GetWindowLongPtr(hListView, GWL_STYLE) & (LVS_NOSORTHEADER as LONG_PTR)) == 0
        && iSortingColumn >= 0
    {
        let mut bSortAscending: BOOL;
        let mut SortInfo: SORT_INFO = core::mem::zeroed();

        let hHeader: HWND = ListView_GetHeader!(hListView);
        let mut hColumn: HDITEM = core::mem::zeroed();

        /* If we are sorting according to another column, uninitialize the old one */
        if iSortedColumn >= 0 && iSortingColumn != iSortedColumn {
            hColumn.mask = HDI_FORMAT;
            Header_GetItem!(hHeader, iSortedColumn, core::ptr::addr_of_mut!(hColumn));
            hColumn.fmt &= (!(HDF_SORTUP | HDF_SORTDOWN)) as i32;
            Header_SetItem!(hHeader, iSortedColumn, core::ptr::addr_of_mut!(hColumn));
        }

        /* Get the sorting state of the new column */
        hColumn.mask = HDI_FORMAT;
        Header_GetItem!(hHeader, iSortingColumn, core::ptr::addr_of_mut!(hColumn));

        /*
         * Check whether we are sorting the list because the user clicked
         * on a column, or because we are refreshing the list:
         *
         * iSortedColumn >= 0 - User clicked on a column; holds the
         *                      old sorting column index.
         * iSortedColumn  < 0 - List being refreshed.
         */
        if iSortedColumn >= 0 {
            /* Invert the sorting direction */
            bSortAscending = ((hColumn.fmt & (HDF_SORTUP as i32)) == 0) as BOOL;
        } else {
            /*
             * If the sorting state of the column is uninitialized,
             * initialize it by default to ascending sorting.
             */
            if (hColumn.fmt & ((HDF_SORTUP | HDF_SORTDOWN) as i32)) == 0 {
                hColumn.fmt |= (HDF_SORTUP as i32);
            }

            /* Keep the same sorting direction */
            bSortAscending = ((hColumn.fmt & (HDF_SORTUP as i32)) != 0) as BOOL;
        }

        /* Set the new column sorting state */
        hColumn.fmt &= (!(if bSortAscending != 0 { HDF_SORTDOWN } else { HDF_SORTUP })) as i32;
        hColumn.fmt |= (if bSortAscending != 0 { HDF_SORTUP } else { HDF_SORTDOWN }) as i32;
        Header_SetItem!(hHeader, iSortingColumn, core::ptr::addr_of_mut!(hColumn));

        /* Sort the list */
        SortInfo.iSortingColumn = iSortingColumn;
        SortInfo.bSortAscending = bSortAscending;
        return ListView_SortItems!(
            hListView,
            Some(CompareFunc),
            core::ptr::addr_of_mut!(SortInfo) as LPARAM,
        );
    } else {
        return (TRUE) as BOOL;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ListWndNotifyProc(hWnd: HWND, wParam: WPARAM, lParam: LPARAM, Result: *mut BOOL) -> BOOL {
    let mut Info: *mut NMLVDISPINFO = core::ptr::null_mut();
    let mut iSortingColumn: core::ffi::c_int = 0;
    UNREFERENCED_PARAMETER!(wParam);
    *Result = (TRUE) as i32;
    match (((*(lParam as LPNMHDR)).code) as i32) as u32 {
        LVN_GETDISPINFO => {
            OnGetDispInfo(lParam as *mut NMLVDISPINFO);
            return (TRUE) as BOOL;
        }
        __c if __c == LVN_COLUMNCLICK as u32 => {
            iSortingColumn = (*(lParam as LPNMLISTVIEW)).iSubItem;
            let _ = ListView_Sort(hWnd, iSortingColumn, g_iSortedColumn);
            g_iSortedColumn = iSortingColumn;
            return (TRUE) as BOOL;
        }
        NM_DBLCLK | NM_RETURN => {
            {
                SendMessageW(
                    hFrameWnd,
                    WM_COMMAND,
                    MAKEWPARAM!(ID_EDIT_MODIFY, 0),
                    0,
                );
            }
            return (TRUE) as BOOL;
        }
        NM_SETFOCUS => {
            (*g_pChildWnd).nFocusPanel = 1;
        }
        LVN_BEGINLABELEDIT => {
            Info = lParam as *mut NMLVDISPINFO;
            if !Info.is_null() {
                let lineinfo: PLINE_INFO = (*Info).item.lParam as PLINE_INFO;
                if (*lineinfo).name.is_null()
                    || wcscmp((*lineinfo).name, RUSTCONV_WSTR_96A296D224F2.as_ptr()) == 0
                {
                    *Result = (TRUE) as i32;
                } else {
                    *Result = (FALSE) as i32;
                }
            } else {
                *Result = (TRUE) as i32;
            }
            return (TRUE) as BOOL;
        }
        LVN_ENDLABELEDIT => {
            Info = lParam as *mut NMLVDISPINFO;
            if !Info.is_null() && !(*Info).item.pszText.is_null() {
                let lineinfo: PLINE_INFO = (*Info).item.lParam as PLINE_INFO;
                if (*lineinfo).name.is_null()
                    || wcscmp((*lineinfo).name, RUSTCONV_WSTR_96A296D224F2.as_ptr()) == 0
                {
                    *Result = (FALSE) as i32;
                } else {
                    if wcslen((*Info).item.pszText) == 0 {
                        let mut msg: [WCHAR; 128] = [0; 128];
                        let mut caption: [WCHAR; 128] =
                            [0; 128];

                        LoadStringW(
                            hInst,
                            IDS_ERR_RENVAL_TOEMPTY,
                            msg.as_mut_ptr(),
                            (ARRAY_SIZE!(msg)) as i32,
                        );
                        LoadStringW(
                            hInst,
                            IDS_ERR_RENVAL_CAPTION,
                            caption.as_mut_ptr(),
                            (ARRAY_SIZE!(caption)) as i32,
                        );
                        MessageBoxW(
                            core::ptr::null_mut(),
                            msg.as_ptr(),
                            caption.as_ptr(),
                            0,
                        );
                        *Result = (TRUE) as i32;
                    } else {
                        let mut hKeyRoot: HKEY = core::ptr::null_mut();
                        let mut keyPath: LPCWSTR = core::ptr::null();
                        let mut lResult: LONG = 0;

                        keyPath = GetItemPath(
                            (*g_pChildWnd).hTreeWnd,
                            core::ptr::null_mut(),
                            core::ptr::addr_of_mut!(hKeyRoot),
                        );
                        lResult = RenameValue(
                            hKeyRoot,
                            keyPath,
                            (*Info).item.pszText,
                            (*lineinfo).name,
                        );
                        (*lineinfo).name = realloc(
                            (*lineinfo).name as *mut core::ffi::c_void,
                            (wcslen((*Info).item.pszText) + 1)
                                * core::mem::size_of::<WCHAR>(),
                        ) as LPWSTR;
                        if !(*lineinfo).name.is_null() {
                            wcscpy((*lineinfo).name, (*Info).item.pszText);
                        }

                        *Result = (TRUE) as i32;
                        return ((lResult) as u32 == ERROR_SUCCESS) as BOOL;
                    }
                }
            } else {
                *Result = (TRUE) as i32;
            }

            return (TRUE) as BOOL;
        }
        _ => {}
    }
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn CreateListView(hwndParent: HWND, id: HMENU, cx: INT) -> HWND {
    let mut rcClient: RECT = core::mem::zeroed();
    let mut hwndLV: HWND;

    /* Get the dimensions of the parent window's client area, and create the list view control. */
    GetClientRect(hwndParent, core::ptr::addr_of_mut!(rcClient));
    hwndLV = CreateWindowExW(WS_EX_CLIENTEDGE, WC_LISTVIEW, RUSTCONV_WSTR_CE539A27F3CF.as_ptr(),
                             WS_VISIBLE | WS_CHILD | WS_TABSTOP | LVS_REPORT | LVS_EDITLABELS | LVS_SHOWSELALWAYS,
                             0, 0, rcClient.right, rcClient.bottom,
                             hwndParent, id, hInst, core::ptr::null_mut());
    if hwndLV.is_null() {
        return core::ptr::null_mut();
    }

    /* Initialize the image list, and add items to the control. */
    'fail: {
        if CreateListColumns(hwndLV, cx) == 0 {
            break 'fail;
        }
        if InitListViewImageLists(hwndLV) == 0 {
            break 'fail;
        }
        return hwndLV;
    }
    DestroyWindow(hwndLV);
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn DestroyListView(hwndLV: HWND) {
    let mut count: INT = 0;
    let mut i: INT = 0;
    let mut item: LVITEMW = core::mem::zeroed();

    count = ListView_GetItemCount!(hwndLV);
    i = 0;
    while i < count {
        item.mask = LVIF_PARAM;
        item.iItem = i;
        let _ = ListView_GetItem!(hwndLV, core::ptr::addr_of_mut!(item));
        free((*(item.lParam as *mut LINE_INFO)).name as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, item.lParam as *mut core::ffi::c_void);
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn RefreshListView(hwndLV: HWND, hKey: HKEY, keyPath: LPCWSTR, bSelectNone: BOOL) -> BOOL {
    let mut max_sub_key_len: DWORD = 0;
    let mut max_val_name_len: DWORD = 0;
    let mut max_val_size: DWORD = 0;
    let mut val_count: DWORD = 0;
    let mut hNewKey: HKEY = core::ptr::null_mut();
    let mut errCode: LONG = 0;
    let mut i: INT = 0;
    let mut c: INT;
    let mut AddedDefault: BOOL = (FALSE) as i32;

    if hwndLV.is_null() {
        return (FALSE) as BOOL;
    }

    let _ = ListView_EditLabel!(hwndLV, -(1));

    SendMessageW(hwndLV, WM_SETREDRAW, (FALSE) as WPARAM, 0);
    DestroyListView(hwndLV);

    let _ = ListView_DeleteAllItems!(hwndLV);

    if hKey.is_null() {
        return (FALSE) as BOOL;
    }

    errCode = RegOpenKeyExW(hKey, keyPath, 0, KEY_READ, core::ptr::addr_of_mut!(hNewKey));
    if (errCode) as u32 != ERROR_SUCCESS {
        return (FALSE) as BOOL;
    }

    /* get size information and resize the buffers if necessary */
    errCode = RegQueryInfoKeyW(
        hNewKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(max_sub_key_len),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(val_count),
        core::ptr::addr_of_mut!(max_val_name_len),
        core::ptr::addr_of_mut!(max_val_size),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if (errCode) as u32 == ERROR_SUCCESS {
        max_val_name_len = max_val_name_len.wrapping_add(1);
        let ValName: *mut WCHAR = HeapAlloc(
            GetProcessHeap(),
            0,
            (max_val_name_len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as *mut WCHAR;
        let mut dwValNameLen: DWORD = max_val_name_len;
        let ValBuf: *mut BYTE = HeapAlloc(
            GetProcessHeap(),
            0,
            (max_val_size as SIZE_T).wrapping_add(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as *mut BYTE;
        let mut dwValSize: DWORD = max_val_size;
        let mut dwIndex: DWORD = 0;
        let mut dwValType: DWORD = 0;
        /*                if (RegQueryValueExW(hNewKey, NULL, NULL, &dwValType, ValBuf, &dwValSize) == ERROR_SUCCESS) { */
        /*                    AddEntryToList(hwndLV, L"(Default)", dwValType, ValBuf, dwValSize); */
        /*                } */
        /*                dwValSize = max_val_size; */
        while (RegEnumValueW(
            hNewKey,
            dwIndex,
            ValName,
            core::ptr::addr_of_mut!(dwValNameLen),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwValType),
            ValBuf,
            core::ptr::addr_of_mut!(dwValSize),
        )) as u32 == ERROR_SUCCESS
        {
            /* Add a terminating 0 character. Usually this is only necessary for strings. */
            let term_val = 0 as BYTE;
            *ValBuf.add((dwValSize + 1) as usize) = term_val;
            *ValBuf.add(dwValSize as usize) = term_val;

            AddEntryToList(
                hwndLV,
                ValName,
                dwValType,
                ValBuf as *mut core::ffi::c_void,
                dwValSize,
                -(1),
                (TRUE) as BOOL,
            );
            dwValNameLen = max_val_name_len;
            dwValSize = max_val_size;
            dwValType = 0;
            dwIndex = dwIndex.wrapping_add(1);
            if wcscmp(ValName, RUSTCONV_WSTR_96A296D224F2.as_ptr()) == 0 {
                AddedDefault = (TRUE) as i32;
            }
        }
        HeapFree(GetProcessHeap(), 0, ValBuf as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, ValName as *mut core::ffi::c_void);
    }
    RegCloseKey(hNewKey);

    if AddedDefault == 0 {
        AddEntryToList(
            hwndLV,
            RUSTCONV_WSTR_96A296D224F2.as_ptr().cast_mut(),
            REG_SZ,
            core::ptr::null_mut(),
            0,
            0,
            (FALSE) as BOOL,
        );
    }
    c = ListView_GetItemCount!(hwndLV);
    i = 0;
    while i < c {
        let _ = ListView_SetItemState!(hwndLV, i, 0, LVIS_FOCUSED | LVIS_SELECTED);
        i += 1;
    }

    if bSelectNone != 0 {
        iListViewSelect = -(1);
    }
    let _ = ListView_SetItemState!(
        hwndLV,
        iListViewSelect,
        LVIS_FOCUSED | LVIS_SELECTED,
        LVIS_FOCUSED | LVIS_SELECTED,
    );
    let _ = ListView_Sort(hwndLV, g_iSortedColumn, -(1));
    SendMessageW(hwndLV, WM_SETREDRAW, (TRUE) as WPARAM, 0);

    return (TRUE) as BOOL;
}
