use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut url_templates: [DWORD; 6] = [
    URLTEMPLATE_CUSTOM as ::core::ffi::c_int as DWORD,
    URLTEMPLATE_LOW as ::core::ffi::c_int as DWORD,
    URLTEMPLATE_MEDLOW as ::core::ffi::c_int as DWORD,
    URLTEMPLATE_MEDIUM as ::core::ffi::c_int as DWORD,
    URLTEMPLATE_MEDHIGH as ::core::ffi::c_int as DWORD,
    URLTEMPLATE_HIGH as ::core::ffi::c_int as DWORD,
];

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct secdlg_data_s {
    pub hsec: HWND,
    pub hlv: HWND,
    pub htb: HWND,
    pub sec_mgr: *mut IInternetSecurityManager,
    pub zone_mgr: *mut IInternetZoneManager,
    pub zone_enumerator: DWORD,
    pub num_zones: DWORD,
    pub zone_attr: *mut ZONEATTRIBUTES,
    pub zones: *mut DWORD,
    pub levels: *mut DWORD,
    pub himages: HIMAGELIST,
    pub last_lv_index: DWORD,
    pub last_level: DWORD,
}

#[allow(non_camel_case_types)]
pub type secdlg_data = secdlg_data_s;

pub const NUM_TRACKBAR_POS: i32 = 5;

// urlmon.h URLTEMPLATE_* values
pub const URLTEMPLATE_CUSTOM: DWORD = 0x00000000;
pub const URLTEMPLATE_LOW: DWORD = 0x00010000;
pub const URLTEMPLATE_MEDLOW: DWORD = 0x00010500;
pub const URLTEMPLATE_MEDIUM: DWORD = 0x00011000;
pub const URLTEMPLATE_MEDHIGH: DWORD = 0x00011500;
pub const URLTEMPLATE_HIGH: DWORD = 0x00012000;

// WINE_DEFAULT_DEBUG_CHANNEL(inetcpl) expansion: debug channel name

unsafe extern "C" fn index_from_urltemplate(value: URLTEMPLATE) -> DWORD {
    let mut index: DWORD = ARRAY_SIZE!(url_templates) as DWORD;
    while index > 0
        && url_templates[index.wrapping_sub(1) as usize] != value as DWORD
    {
        index = index.wrapping_sub(1);
    }
    index = index.wrapping_sub(1);
    if index == 0 && value as ::core::ffi::c_uint != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/cpl/inetcpl/security.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"index_from_urltemplate\0" as *const u8 as *const ::core::ffi::c_char,
                83 as ::core::ffi::c_int,
                b"URLTEMPLATE 0x%x not supported\n\0" as *const u8 as *const ::core::ffi::c_char,
                value as ::core::ffi::c_uint,
            );
        }
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
            b"dll/cpl/inetcpl/security.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"index_from_urltemplate\0" as *const u8 as *const ::core::ffi::c_char,
            85 as ::core::ffi::c_int,
            b"URLTEMPLATE 0x%08x=> Level %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            value as ::core::ffi::c_uint,
            index,
        );
    }
    return index;
}

unsafe extern "C" fn update_security_level(sd: *mut secdlg_data, lv_index: DWORD, tb_index: DWORD) {
    let mut name: [WCHAR; 512] = [0; 512];
    let mut current_index: DWORD = 0;

    if (*(*sd).levels.offset(lv_index as isize) != (*sd).last_level) || (tb_index > 0) {
        if *(*sd).levels.offset(lv_index as isize) == 0 || (*sd).last_level == 0 {
            ShowWindow(
                (*sd).htb,
                if *(*sd).levels.offset(lv_index as isize) != 0 {
                    (SW_NORMAL) as i32
                } else {
                    (SW_HIDE) as i32
                },
            );
        }

        current_index = if tb_index > 0 {
            tb_index
        } else {
            index_from_urltemplate(*(*sd).levels.offset(lv_index as isize) as URLTEMPLATE)
        };

        name[0 as usize] = 0;
        LoadStringW(
            hcpl as HINSTANCE,
            (IDS_SEC_LEVEL0 as DWORD).wrapping_add(current_index) as UINT,
            &raw mut name as LPWSTR,
            ARRAY_SIZE!(name) as ::core::ffi::c_int,
        );
        SetWindowTextW(
            GetDlgItem((*sd).hsec, (IDC_SEC_LEVEL as i32)),
            &raw mut name as *mut WCHAR as LPCWSTR,
        );

        name[0 as usize] = 0;
        LoadStringW(
            hcpl as HINSTANCE,
            (IDS_SEC_LEVEL0_INFO as DWORD).wrapping_add(current_index.wrapping_mul(0x10 as DWORD)) as UINT,
            &raw mut name as LPWSTR,
            ARRAY_SIZE!(name) as ::core::ffi::c_int,
        );
        SetWindowTextW(
            GetDlgItem((*sd).hsec, (IDC_SEC_LEVEL_INFO as i32)),
            &raw mut name as *mut WCHAR as LPCWSTR,
        );

        if current_index != 0 {
            SendMessageW(
                (*sd).htb,
                TBM_SETPOS as UINT,
                TRUE as WPARAM,
                (NUM_TRACKBAR_POS as DWORD).wrapping_sub(current_index) as LPARAM,
            );
        }

        (*sd).last_level = *(*sd).levels.offset(lv_index as isize);
    }
}

unsafe extern "C" fn update_zone_info(sd: *mut secdlg_data, lv_index: DWORD) {
    let za: *mut ZONEATTRIBUTES = (*sd).zone_attr.add(lv_index as usize);
    let mut name: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut len: DWORD;

    SetWindowTextW(
        GetDlgItem((*sd).hsec, (IDC_SEC_ZONE_INFO as i32)),
        core::ptr::addr_of_mut!((*za).szDescription) as *mut WCHAR as LPCWSTR,
    );

    len = LoadStringW(
        hcpl as HINSTANCE,
        IDS_SEC_SETTINGS as UINT,
        core::ptr::addr_of_mut!(name) as *mut WCHAR as LPWSTR,
        ARRAY_SIZE!(name) as ::core::ffi::c_int,
    ) as DWORD;
    lstrcpynW(
        (core::ptr::addr_of_mut!(name) as *mut WCHAR).offset(len as isize) as LPWSTR,
        core::ptr::addr_of_mut!((*za).szDisplayName) as *mut WCHAR as LPCWSTR,
        (ARRAY_SIZE!(name) as usize)
            .wrapping_sub(len as usize)
            .wrapping_sub(1 as usize) as ::core::ffi::c_int,
    );

    SetWindowTextW(
        GetDlgItem((*sd).hsec, (IDC_SEC_GROUP as i32)),
        core::ptr::addr_of_mut!(name) as *mut WCHAR as LPCWSTR,
    );

    update_security_level(sd, lv_index, 0);
    (*sd).last_lv_index = lv_index;
}

unsafe extern "C" fn add_zone_to_listview(
    sd: *mut secdlg_data,
    pindex: *mut DWORD,
    zone: DWORD,
) {
    let mut lv_index: DWORD = *pindex;
    let mut za: *mut ZONEATTRIBUTES = (*sd).zone_attr.add(lv_index as usize);
    let mut lvitem: LVITEMW = core::mem::zeroed();
    let mut hr: HRESULT = 0;
    let mut iconid: INT = 0 as INT;
    let mut hdll: HMODULE = core::ptr::null_mut();
    let mut ptr: *mut WCHAR = core::ptr::null_mut();
    let mut icon: HICON = core::ptr::null_mut();

    *(*sd).zones.add(lv_index as usize) = zone;

    memset(
        core::ptr::addr_of_mut!(lvitem) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<LVITEMW>() as size_t,
    );
    memset(
        za as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<ZONEATTRIBUTES>() as size_t,
    );
    (*za).cbSize = ::core::mem::size_of::<ZONEATTRIBUTES>() as ULONG;
    hr = IInternetZoneManager_GetZoneAttributes((*sd).zone_mgr, zone, za);
    if hr >= 0 as HRESULT {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/cpl/inetcpl/security.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"add_zone_to_listview\0" as *const u8 as *const ::core::ffi::c_char,
                171 as ::core::ffi::c_int,
                b"displayname: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_w(core::ptr::addr_of_mut!((*za).szDisplayName) as *mut WCHAR),
            );
        }
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/cpl/inetcpl/security.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"add_zone_to_listview\0" as *const u8 as *const ::core::ffi::c_char,
                172 as ::core::ffi::c_int,
                b"description: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_w(core::ptr::addr_of_mut!((*za).szDescription) as *mut WCHAR),
            );
        }
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/cpl/inetcpl/security.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"add_zone_to_listview\0" as *const u8 as *const ::core::ffi::c_char,
                174 as ::core::ffi::c_int,
                b"minlevel: 0x%x, recommended: 0x%x, current: 0x%x (flags: 0x%x)\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                (*za).dwTemplateMinLevel,
                (*za).dwTemplateRecommended,
                (*za).dwTemplateCurrentLevel,
                (*za).dwFlags,
            );
        }

        if ((*za).dwFlags & ZAFLAGS_NO_UI as DWORD) != 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/cpl/inetcpl/security.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"add_zone_to_listview\0" as *const u8 as *const ::core::ffi::c_char,
                    177 as ::core::ffi::c_int,
                    b"item %d (zone %d): UI disabled for %s\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    lv_index,
                    zone,
                    debugstr_w(core::ptr::addr_of_mut!((*za).szDisplayName) as *mut WCHAR),
                );
            }
            return;
        }

        *(*sd).levels.add(lv_index as usize) = (*za).dwTemplateCurrentLevel;

        lvitem.mask = (LVIF_TEXT | LVIF_IMAGE | LVIF_PARAM) as UINT;
        lvitem.iItem = lv_index as ::core::ffi::c_int;
        lvitem.iSubItem = 0 as ::core::ffi::c_int;
        lvitem.pszText = core::ptr::addr_of_mut!((*za).szDisplayName) as *mut WCHAR as LPWSTR;
        lvitem.lParam = zone as LPARAM;

        /* format is "filename.ext#iconid" */
        ptr = StrChrW(
            core::ptr::addr_of_mut!((*za).szIconPath) as *mut WCHAR as LPCWSTR,
            '#' as WCHAR,
        ) as *mut WCHAR;
        if !ptr.is_null() {
            *ptr = 0 as WCHAR;
            ptr = ptr.add(1);
            iconid = StrToIntW(ptr as LPCWSTR) as INT;
            hdll = LoadLibraryExW(
                core::ptr::addr_of_mut!((*za).szIconPath) as *mut WCHAR as LPCWSTR,
                core::ptr::null_mut(),
                LOAD_LIBRARY_AS_DATAFILE as DWORD,
            ) as HMODULE;
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/cpl/inetcpl/security.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"add_zone_to_listview\0" as *const u8 as *const ::core::ffi::c_char,
                    196 as ::core::ffi::c_int,
                    b"%p: icon #%d from %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    hdll,
                    iconid,
                    debugstr_w(core::ptr::addr_of_mut!((*za).szIconPath) as *mut WCHAR),
                );
            }
            icon = LoadImageW(
                hdll as HINSTANCE,
                MAKEINTRESOURCEW((iconid as WORD) as _) as LPCWSTR,
                IMAGE_ICON as UINT,
                GetSystemMetrics(SM_CXICON as ::core::ffi::c_int),
                GetSystemMetrics(SM_CYICON as ::core::ffi::c_int),
                LR_SHARED as UINT,
            ) as HICON;

            if icon.is_null() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int)
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/cpl/inetcpl/security.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"add_zone_to_listview\0" as *const u8 as *const ::core::ffi::c_char,
                        202 as ::core::ffi::c_int,
                        b"item %d (zone %d): missing icon #%d in %s\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        lv_index,
                        zone,
                        iconid,
                        debugstr_w(core::ptr::addr_of_mut!((*za).szIconPath) as *mut WCHAR),
                    );
                }
            }

            /* the failure result (NULL) from LoadImageW let ImageList_AddIcon fail
               with -1, which is reused in ListView_InsertItemW to disable the image */
            lvitem.iImage = ImageList_AddIcon!((*sd).himages, icon) as ::core::ffi::c_int;
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int)
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/cpl/inetcpl/security.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"add_zone_to_listview\0" as *const u8 as *const ::core::ffi::c_char,
                    210 as ::core::ffi::c_int,
                    b"item %d (zone %d): malformed szIconPath %s\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    lv_index,
                    zone,
                    debugstr_w(core::ptr::addr_of_mut!((*za).szIconPath) as *mut WCHAR),
                );
            }
        }

        if ListView_InsertItemW!((*sd).hlv, core::ptr::addr_of_mut!(lvitem))
            >= 0 as ::core::ffi::c_int
        {
            /* activate first item in the listview */
            if lv_index == 0 {
                lvitem.state = (LVIS_FOCUSED | LVIS_SELECTED) as UINT;
                lvitem.stateMask = (LVIS_FOCUSED | LVIS_SELECTED) as UINT;
                SendMessageW(
                    (*sd).hlv,
                    LVM_SETITEMSTATE as UINT,
                    0 as WPARAM,
                    core::ptr::addr_of_mut!(lvitem) as LPARAM,
                );
                (*sd).last_level = !(0 as DWORD);
                update_zone_info(sd, lv_index);
            }
            *pindex = (*pindex).wrapping_add(1);
        }
        FreeLibrary(hdll);
    } else {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int)
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/cpl/inetcpl/security.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"add_zone_to_listview\0" as *const u8 as *const ::core::ffi::c_char,
                226 as ::core::ffi::c_int,
                b"item %d (zone %d): GetZoneAttributes failed with 0x%x\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                lv_index,
                zone,
                hr,
            );
        }
    }
}

unsafe extern "C" fn security_cleanup_zones(sd: *mut secdlg_data) {
    if (*sd).zone_enumerator != 0 {
        IInternetZoneManager_DestroyZoneEnumerator((*sd).zone_mgr, (*sd).zone_enumerator);
    }

    if !(*sd).zone_mgr.is_null() {
        IInternetZoneManager_Release((*sd).zone_mgr);
    }

    if !(*sd).sec_mgr.is_null() {
        IInternetSecurityManager_Release((*sd).sec_mgr);
    }
}

unsafe extern "C" fn security_enum_zones(sd: *mut secdlg_data) -> HRESULT {
    let mut hr: HRESULT = 0;
    hr = CoInternetCreateSecurityManager(
        core::ptr::null_mut(),
        &raw mut (*sd).sec_mgr,
        0,
    );
    if hr >= 0 {
        hr = CoInternetCreateZoneManager(
            core::ptr::null_mut(),
            &raw mut (*sd).zone_mgr,
            0,
        );
        if hr >= 0 {
            hr = IInternetZoneManager_CreateZoneEnumerator(
                (*sd).zone_mgr,
                &raw mut (*sd).zone_enumerator,
                &raw mut (*sd).num_zones,
                0,
            );
        }
    }
    return hr;
}

unsafe extern "C" fn security_on_destroy(sd: *mut secdlg_data) -> INT_PTR {
    heap_free((*sd).zone_attr as *mut core::ffi::c_void);
    heap_free((*sd).zones as *mut core::ffi::c_void);
    // The C's image-list teardown is `#ifndef __REACTOS__`: the listview has no
    // LVS_SHAREIMAGELISTS, so comctl32 already destroyed it with the window.

    security_cleanup_zones(sd);
    SetWindowLongPtrW((*sd).hsec, DWLP_USER, 0);
    heap_free(sd as *mut core::ffi::c_void);
    return (TRUE) as INT_PTR;
}

unsafe extern "C" fn security_on_initdialog(hsec: HWND) -> INT_PTR {
    let mut sd: *mut secdlg_data = core::ptr::null_mut();
    let mut hr: HRESULT = 0;
    let mut current_zone: DWORD = 0;
    let mut lv_index: DWORD = 0;
    let mut i: DWORD = 0;

    sd = heap_alloc_zero((core::mem::size_of::<secdlg_data>() as SIZE_T) as _) as *mut secdlg_data;
    SetWindowLongPtrW(hsec, DWLP_USER as ::core::ffi::c_int, sd as LONG_PTR);
    if sd.is_null() {
        return FALSE as INT_PTR;
    }

    (*sd).hsec = hsec;
    (*sd).hlv = GetDlgItem(hsec, IDC_SEC_LISTVIEW as ::core::ffi::c_int);
    (*sd).htb = GetDlgItem(hsec, IDC_SEC_TRACKBAR as ::core::ffi::c_int);

    EnableWindow((*sd).htb, (FALSE) as BOOL); /* not changeable yet */

    SendMessageW(
        (*sd).htb,
        TBM_SETRANGE as UINT,
        FALSE as WPARAM,
        MAKELONG!(0, NUM_TRACKBAR_POS - 1) as LPARAM,
    );
    SendMessageW(
        (*sd).htb,
        TBM_SETTICFREQ as UINT,
        1 as WPARAM,
        0 as LPARAM,
    );

    /* Create the image lists for the listview */
    (*sd).himages = ImageList_Create(
        GetSystemMetrics(SM_CXICON as ::core::ffi::c_int),
        GetSystemMetrics(SM_CYICON as ::core::ffi::c_int),
        (ILC_COLOR32 | ILC_MASK) as UINT,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );

    if (*sd).himages.is_null() {
        return FALSE as INT_PTR;
    }
    SendMessageW(
        (*sd).hlv,
        LVM_SETIMAGELIST as UINT,
        LVSIL_NORMAL as WPARAM,
        (*sd).himages as LPARAM,
    );

    hr = security_enum_zones(sd);
    if hr < 0 {
        security_on_destroy(sd);
        return FALSE as INT_PTR;
    }

    /* remember ZONEATTRIBUTES for a listview entry */
    (*sd).zone_attr = heap_alloc(
        ((core::mem::size_of::<ZONEATTRIBUTES>() as SIZE_T).wrapping_mul((*sd).num_zones as SIZE_T)) as _,
    ) as *mut ZONEATTRIBUTES;
    if (*sd).zone_attr.is_null() {
        security_on_destroy(sd);
        return FALSE as INT_PTR;
    }

    /* remember zone number and current security level for a listview entry */
    (*sd).zones = heap_alloc(
        ((core::mem::size_of::<DWORD>() as SIZE_T)
            .wrapping_add(core::mem::size_of::<DWORD>() as SIZE_T)
            .wrapping_mul((*sd).num_zones as SIZE_T)) as _,
    ) as *mut DWORD;
    if (*sd).zones.is_null() {
        security_on_destroy(sd);
        return FALSE as INT_PTR;
    }
    (*sd).levels = (*sd).zones.offset((*sd).num_zones as isize);

    /* use the same order as visible with native inetcpl.cpl */
    add_zone_to_listview(sd, core::ptr::addr_of_mut!(lv_index), URLZONE_INTERNET as DWORD);
    add_zone_to_listview(sd, core::ptr::addr_of_mut!(lv_index), URLZONE_INTRANET as DWORD);
    add_zone_to_listview(sd, core::ptr::addr_of_mut!(lv_index), URLZONE_TRUSTED as DWORD);
    add_zone_to_listview(sd, core::ptr::addr_of_mut!(lv_index), URLZONE_UNTRUSTED as DWORD);

    i = 0 as DWORD;
    while i < (*sd).num_zones {
        hr = IInternetZoneManager_GetZoneAt(
            (*sd).zone_mgr,
            (*sd).zone_enumerator,
            i,
            core::ptr::addr_of_mut!(current_zone),
        );
        if hr >= 0 && current_zone != URLZONE_INVALID as DWORD {
            if current_zone == 0 || current_zone > URLZONE_UNTRUSTED as DWORD {
                add_zone_to_listview(sd, core::ptr::addr_of_mut!(lv_index), current_zone);
            }
        }
        i = i.wrapping_add(1);
    }
    return TRUE as INT_PTR;
}

unsafe extern "C" fn security_on_notify(
    mut sd: *mut secdlg_data,
    mut wparam: WPARAM,
    mut lparam: LPARAM,
) -> INT_PTR {
    let mut nm: *mut NMLISTVIEW = ::core::ptr::null_mut::<NMLISTVIEW>();
    nm = lparam as *mut NMLISTVIEW;
    match ((*nm).hdr.code) as i32 {
        LVN_ITEMCHANGED => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/cpl/inetcpl/security.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"security_on_notify\0" as *const u8 as *const ::core::ffi::c_char,
                    389 as ::core::ffi::c_int,
                    b"LVN_ITEMCHANGED (0x%lx, 0x%lx) from %p with code: %d (item: %d, uNewState: %u)\n\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    wparam,
                    lparam,
                    (*nm).hdr.hwndFrom,
                    (*nm).hdr.code,
                    (*nm).iItem,
                    (*nm).uNewState,
                );
            }
            if ((*nm).uNewState & LVIS_SELECTED as UINT) == LVIS_SELECTED as UINT {
                update_zone_info(sd, (*nm).iItem as DWORD);
            }
        }
        PSN_APPLY => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/cpl/inetcpl/security.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"security_on_notify\0" as *const u8 as *const ::core::ffi::c_char,
                    397 as ::core::ffi::c_int,
                    b"PSN_APPLY (0x%lx, 0x%lx) from %p with code: %d\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    wparam,
                    lparam,
                    (*nm).hdr.hwndFrom,
                    (*nm).hdr.code,
                );
            }
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/cpl/inetcpl/security.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"security_on_notify\0" as *const u8 as *const ::core::ffi::c_char,
                    402 as ::core::ffi::c_int,
                    b"WM_NOTIFY (0x%lx, 0x%lx) from %p with code: %d\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    wparam,
                    lparam,
                    (*nm).hdr.hwndFrom,
                    (*nm).hdr.code,
                );
            }
        }
    }
    return FALSE as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn security_dlgproc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> INT_PTR {
    let mut sd: *mut secdlg_data;

    if msg == WM_INITDIALOG as UINT {
        return security_on_initdialog(hwnd);
    }

    sd = GetWindowLongPtrW(hwnd, DWLP_USER as core::ffi::c_int) as *mut secdlg_data;
    if !sd.is_null() {
        match msg {
            WM_NOTIFY => {
                return security_on_notify(sd, wparam, lparam);
            }
            WM_NCDESTROY => {
                return security_on_destroy(sd);
            }
            _ => {
                /* do not flood the log */
                if (msg == WM_SETCURSOR as UINT)
                    || (msg == WM_NCHITTEST as UINT)
                    || (msg == WM_MOUSEMOVE as UINT)
                    || (msg == WM_MOUSEACTIVATE as UINT)
                    || (msg == WM_PARENTNOTIFY as UINT)
                {
                    return (FALSE) as INT_PTR;
                }
                if ((*__wine_dbch___default).flags as core::ffi::c_int
                    & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int)
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/cpl/inetcpl/security.c\0"
                            as *const u8 as *const core::ffi::c_char,
                        b"security_dlgproc\0" as *const u8 as *const core::ffi::c_char,
                        436 as core::ffi::c_int,
                        b"(%p, 0x%08x/%03d, 0x%08lx, 0x%08lx)\n\0" as *const u8
                            as *const core::ffi::c_char,
                        hwnd,
                        msg,
                        msg,
                        wparam,
                        lparam,
                    );
                }
            }
        }
    }
    return (FALSE) as INT_PTR;
}
