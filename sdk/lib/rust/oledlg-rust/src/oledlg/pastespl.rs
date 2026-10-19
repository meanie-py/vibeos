use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut ps_flags: [ps_flag; 9] = [
    ps_flag {
        flag: 0x1 as DWORD,
        name: b"PSF_SHOWHELP\0" as *const u8 as *const ::core::ffi::c_char,
    },
    ps_flag {
        flag: 0x2 as DWORD,
        name: b"PSF_SELECTPASTE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    ps_flag {
        flag: 0x4 as DWORD,
        name: b"PSF_SELECTPASTELINK\0" as *const u8 as *const ::core::ffi::c_char,
    },
    ps_flag {
        flag: 0x8 as DWORD,
        name: b"PSF_CHECKDISPLAYASICON\0" as *const u8 as *const ::core::ffi::c_char,
    },
    ps_flag {
        flag: 0x10 as DWORD,
        name: b"PSF_DISABLEDISPLAYASICON\0" as *const u8 as *const ::core::ffi::c_char,
    },
    ps_flag {
        flag: 0x20 as DWORD,
        name: b"PSF_HIDECHANGEICON\0" as *const u8 as *const ::core::ffi::c_char,
    },
    ps_flag {
        flag: 0x40 as DWORD,
        name: b"PSF_STAYONCLIPBOARDCHANGE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    ps_flag {
        flag: 0x80 as DWORD,
        name: b"PSF_NOREFRESHDATAOBJECT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    ps_flag {
        flag: -(1 as ::core::ffi::c_int) as DWORD,
        name: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];

// COBJMACROS: object-like macro with no value, nothing to emit.
// WINE_DEFAULT_DEBUG_CHANNEL(ole): debug-channel macro, not emitted.

#[repr(C)]
pub struct ps_struct_t {
    pub ps: *mut OLEUIPASTESPECIALW,
    pub flags: DWORD,
    pub source_name: *mut WCHAR,
    pub link_source_name: *mut WCHAR,
    pub type_name: *mut WCHAR,
    pub link_type_name: *mut WCHAR,
    pub app_name: LPOLESTR,
}
pub type ps_struct_t_ptr = *mut ps_struct_t;

#[repr(C)]
pub struct ps_flag {
    pub flag: DWORD,
    pub name: *const i8,
}

unsafe impl Sync for ps_flag {}

unsafe extern "C" fn dump_ps_flags(mut flags: DWORD) {
    let mut flagstr: [::core::ffi::c_char; 1000] = [0; 1000];
    let mut flag: *const ps_flag = &raw const ps_flags as *const ps_flag;
    while !(*flag).name.is_null() {
        if (flags & (*flag).flag) != 0 {
            strcat(
                &raw mut flagstr as *mut ::core::ffi::c_char,
                (*flag).name,
            );
            strcat(
                &raw mut flagstr as *mut ::core::ffi::c_char,
                b"|\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        flag = flag.offset(1);
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
            b"dll/win32/oledlg/pastespl.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"dump_ps_flags\0" as *const u8 as *const ::core::ffi::c_char,
            80 as ::core::ffi::c_int,
            b"flags %08lx %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            flags,
            &raw mut flagstr as *mut ::core::ffi::c_char,
        );
    }
}

unsafe extern "C" fn dump_pastespecial(ps: *const OLEUIPASTESPECIALW) {
    let mut i: INT = 0;
    let mut j: UINT = 0;

    dump_ps_flags((*ps).dwFlags);

    if IS_INTRESOURCE!((*ps).lpszTemplate) {
        TRACE!("hinst %p template %04x hresource %p\n", (*ps).hInstance, ((*ps).lpszTemplate as ULONG_PTR as WORD) as core::ffi::c_uint, (*ps).hResource);
    } else {
        TRACE!("hinst %p template %s hresource %p\n", (*ps).hInstance, debugstr_w((*ps).lpszTemplate as *const WCHAR), (*ps).hResource);
    }

    while i < (*ps).cPasteEntries {
        i += 1;
    }

    i = 0;
    while i < (*ps).cLinkTypes {
        TRACE!("arrLinkTypes[%d] %08x\n", i, *(*ps).arrLinkTypes.offset(i as isize));
        i += 1;
    }

    while j < (*ps).cClsidExclude {
        TRACE!("lpClsidExclude[%u] %s\n", j, debugstr_guid((*ps).lpClsidExclude.offset(j as isize) as *mut GUID));
        j = j.wrapping_add(1);
    }
}

unsafe extern "C" fn strdupAtoW(str: *const core::ffi::c_char) -> *mut WCHAR {
    let mut len: DWORD = 0;
    let mut ret: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    if str.is_null() {
        return core::ptr::null_mut::<WCHAR>();
    }
    len = MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        str as LPCSTR,
        -(1 as core::ffi::c_int),
        core::ptr::null_mut::<WCHAR>(),
        0 as core::ffi::c_int,
    ) as DWORD;
    ret = malloc(
        (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as *mut WCHAR;
    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        str as LPCSTR,
        -(1 as core::ffi::c_int),
        ret as LPWSTR,
        len as core::ffi::c_int,
    );
    return ret;
}

unsafe extern "C" fn get_descriptors(mut hdlg: HWND, mut ps_struct: *mut ps_struct_t) {
    let mut fmtetc: FORMATETC = tagFORMATETC {
        cfFormat: 0,
        ptd: ::core::ptr::null_mut::<DVTARGETDEVICE>(),
        dwAspect: 0,
        lindex: 0,
        tymed: 0,
    };
    let mut stg: STGMEDIUM = tagSTGMEDIUM {
        tymed: 0,
        __bindgen_anon_1: tagSTGMEDIUM__bindgen_ty_1 { hBitmap: ::core::ptr::null_mut::<HBITMAP__>(),
        },
        pUnkForRelease: ::core::ptr::null_mut::<IUnknown>(),
    };
    fmtetc.tymed = TYMED_HGLOBAL as DWORD;
    fmtetc.dwAspect = DVASPECT_CONTENT as DWORD;
    fmtetc.ptd = ::core::ptr::null_mut::<DVTARGETDEVICE>();
    fmtetc.lindex = -(1 as LONG);

    fmtetc.cfFormat = cf_object_descriptor as CLIPFORMAT;
    if IDataObject_GetData(
        (*(*ps_struct).ps).lpSrcDataObj as *mut IDataObject,
        core::ptr::addr_of_mut!(fmtetc),
        core::ptr::addr_of_mut!(stg),
    ) == S_OK
    {
        let mut obj_desc: *mut OBJECTDESCRIPTOR =
            GlobalLock(stg.__bindgen_anon_1.hGlobal) as *mut OBJECTDESCRIPTOR;
        if (*obj_desc).dwSrcOfCopy != 0 {
            (*ps_struct).source_name = wcsdup(
                (obj_desc as *mut ::core::ffi::c_char).offset((*obj_desc).dwSrcOfCopy as isize)
                    as *mut WCHAR,
            ) as *mut WCHAR;
        }
        if (*obj_desc).dwFullUserTypeName != 0 {
            (*ps_struct).type_name = wcsdup(
                (obj_desc as *mut ::core::ffi::c_char)
                    .offset((*obj_desc).dwFullUserTypeName as isize) as *mut WCHAR,
            ) as *mut WCHAR;
        }
        OleRegGetUserType(
            core::ptr::addr_of_mut!((*obj_desc).clsid),
            USERCLASSTYPE_APPNAME as DWORD,
            core::ptr::addr_of_mut!((*ps_struct).app_name),
        );
        GlobalUnlock(stg.__bindgen_anon_1.hGlobal);
        GlobalFree(stg.__bindgen_anon_1.hGlobal);
    } else {
        /* Try to get some data using some of the other clipboard formats */
    }

    fmtetc.cfFormat = cf_link_src_descriptor as CLIPFORMAT;
    if IDataObject_GetData(
        (*(*ps_struct).ps).lpSrcDataObj as *mut IDataObject,
        core::ptr::addr_of_mut!(fmtetc),
        core::ptr::addr_of_mut!(stg),
    ) == S_OK
    {
        let mut obj_desc_0: *mut OBJECTDESCRIPTOR =
            GlobalLock(stg.__bindgen_anon_1.hGlobal) as *mut OBJECTDESCRIPTOR;
        if (*obj_desc_0).dwSrcOfCopy != 0 {
            (*ps_struct).link_source_name = wcsdup(
                (obj_desc_0 as *mut ::core::ffi::c_char).offset((*obj_desc_0).dwSrcOfCopy as isize)
                    as *mut WCHAR,
            ) as *mut WCHAR;
        }
        if (*obj_desc_0).dwFullUserTypeName != 0 {
            (*ps_struct).link_type_name = wcsdup(
                (obj_desc_0 as *mut ::core::ffi::c_char)
                    .offset((*obj_desc_0).dwFullUserTypeName as isize)
                    as *mut WCHAR,
            ) as *mut WCHAR;
        }
        GlobalUnlock(stg.__bindgen_anon_1.hGlobal);
        GlobalFree(stg.__bindgen_anon_1.hGlobal);
    }

    if (*ps_struct).source_name.is_null() && (*ps_struct).link_source_name.is_null() {
        let mut buf: [WCHAR; 200] = [0; 200];
        LoadStringW(
            OLEDLG_hInstance,
            IDS_PS_UNKNOWN_SRC as UINT,
            core::ptr::addr_of_mut!(buf) as LPWSTR,
            ARRAY_SIZE!(buf) as ::core::ffi::c_int,
        );
        (*ps_struct).source_name =
            wcsdup(core::ptr::addr_of_mut!(buf) as *mut WCHAR) as *mut WCHAR;
    }

    if (*ps_struct).type_name.is_null() && (*ps_struct).link_type_name.is_null() {
        let mut buf: [WCHAR; 200] = [0; 200];
        LoadStringW(
            OLEDLG_hInstance,
            IDS_PS_UNKNOWN_TYPE as UINT,
            core::ptr::addr_of_mut!(buf) as LPWSTR,
            ARRAY_SIZE!(buf) as ::core::ffi::c_int,
        );
        (*ps_struct).type_name =
            wcsdup(core::ptr::addr_of_mut!(buf) as *mut WCHAR) as *mut WCHAR;
    }
}

unsafe extern "C" fn add_entry_to_lb(hdlg: HWND, id: UINT, pe: *mut OLEUIPASTEENTRYW) -> BOOL {
    let hwnd: HWND = GetDlgItem(hdlg, id as ::core::ffi::c_int);
    let mut ret: BOOL = FALSE as BOOL;

    /* FIXME %s handling */

    /* Note that this suffers from the same bug as native, in that if a new string
       is a substring of an already added string, then the FINDSTRING will succeed
       this is probably not what we want */
    if SendMessageW(
        hwnd,
        LB_FINDSTRING as UINT,
        0 as WPARAM,
        (*pe).lpstrFormatName as LPARAM,
    ) == -(1 as LRESULT)
    {
        let pos: LRESULT = SendMessageW(
            hwnd,
            LB_ADDSTRING as UINT,
            0 as WPARAM,
            (*pe).lpstrFormatName as LPARAM,
        );
        SendMessageW(
            hwnd,
            LB_SETITEMDATA as UINT,
            pos as WPARAM,
            pe as LPARAM,
        );
        ret = TRUE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn init_pastelist(hdlg: HWND, ps: *mut OLEUIPASTESPECIALW) -> DWORD {
    let mut penum: *mut IEnumFORMATETC = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut fmts: [FORMATETC; 20] = core::mem::zeroed();
    let mut fetched: DWORD;
    let mut items_added: DWORD = 0;

    hr = IDataObject_EnumFormatEtc(
        (*ps).lpSrcDataObj as *mut IDataObject,
        DATADIR_GET as DWORD,
        core::ptr::addr_of_mut!(penum),
    );
    if hr < 0 {
        WARN!("Unable to create IEnumFORMATETC\n");
        return 0;
    }

    /* The native version grabs only the first 20 fmts and we do the same */
    fetched = 0;
    hr = IEnumFORMATETC_Next(
        penum,
        ARRAY_SIZE!(fmts) as ULONG,
        core::ptr::addr_of_mut!(fmts) as *mut FORMATETC,
        core::ptr::addr_of_mut!(fetched),
    );

    if hr >= 0 {
        let mut src_fmt: DWORD;
        let mut req_fmt: DWORD;
        req_fmt = 0;
        while req_fmt < (*ps).cPasteEntries as DWORD {
            /* This is used by update_structure() to set nSelectedIndex on exit */
            (*(*ps).arrPasteEntries.offset(req_fmt as isize)).dwScratchSpace = req_fmt;

            src_fmt = 0;
            while src_fmt < fetched {
                if (*(*ps).arrPasteEntries.offset(req_fmt as isize)).fmtetc.cfFormat
                    == fmts[src_fmt as usize].cfFormat
                {
                    add_entry_to_lb(
                        hdlg,
                        IDC_PS_PASTELIST as UINT,
                        (*ps).arrPasteEntries.offset(req_fmt as isize),
                    );
                    items_added += 1;
                    break;
                }
                src_fmt += 1;
            }
            req_fmt += 1;
        }
    }

    IEnumFORMATETC_Release(penum);
    EnableWindow(
        GetDlgItem(hdlg, (IDC_PS_PASTE as i32)),
        (items_added != 0) as i32,
    );
    return items_added;
}

unsafe extern "C" fn init_linklist(mut hdlg: HWND, mut ps: *mut OLEUIPASTESPECIALW) -> DWORD {
    let mut hr: HRESULT = 0;
    let mut supported_mask: DWORD = 0 as DWORD;
    let mut items_added: DWORD = 0 as DWORD;
    let mut link: ::core::ffi::c_int = 0;
    let mut req_fmt: ::core::ffi::c_int = 0;
    let mut fmt: FORMATETC = tagFORMATETC {
        cfFormat: 0 as CLIPFORMAT,
        ptd: ::core::ptr::null_mut::<DVTARGETDEVICE>(),
        dwAspect: DVASPECT_CONTENT as ::core::ffi::c_int as DWORD,
        lindex: -(1 as ::core::ffi::c_int) as LONG,
        tymed: -(1 as ::core::ffi::c_int) as DWORD,
    };
    link = 0 as ::core::ffi::c_int;
    while link < (*ps).cLinkTypes && link < (PS_MAXLINKTYPES as i32) {
        fmt.cfFormat = *(*ps).arrLinkTypes.offset(link as isize) as CLIPFORMAT;
        hr = IDataObject_QueryGetData((*ps).lpSrcDataObj as *mut IDataObject, core::ptr::addr_of_mut!(fmt));
        if hr == S_OK {
            supported_mask |= ((1 as ::core::ffi::c_int) << link) as DWORD;
        }
        link += 1;
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
            b"dll/win32/oledlg/pastespl.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"init_linklist\0" as *const u8 as *const ::core::ffi::c_char,
            262 as ::core::ffi::c_int,
            b"supported_mask %02lx\n\0" as *const u8 as *const ::core::ffi::c_char,
            supported_mask,
        );
    }
    req_fmt = 0 as ::core::ffi::c_int;
    while req_fmt < (*ps).cPasteEntries {
        let mut linktypes: DWORD = 0;
        if (*(*ps).arrPasteEntries.offset(req_fmt as isize)).dwFlags
            & OLEUIPASTE_LINKANYTYPE as ::core::ffi::c_int as DWORD
            != 0
        {
            linktypes = 0xff as DWORD;
        } else {
            linktypes = (*(*ps).arrPasteEntries.offset(req_fmt as isize)).dwFlags
                & 0xff as DWORD;
        }
        if linktypes & supported_mask != 0 {
            add_entry_to_lb(
                hdlg,
                IDC_PS_PASTELINKLIST as UINT,
                (*ps).arrPasteEntries.offset(req_fmt as isize),
            );
            items_added = items_added.wrapping_add(1);
        }
        req_fmt += 1;
    }
    EnableWindow(
        GetDlgItem(hdlg, (IDC_PS_PASTELINK as i32)),
        (items_added != 0 as DWORD) as ::core::ffi::c_int,
    );
    return items_added;
}

unsafe extern "C" fn update_display_list(mut hdlg: HWND, mut src_list_id: UINT) {
    let mut count: LONG = 0;
    let mut i: LONG = 0;
    let mut old_pos: LONG = 0;
    let mut txt: [WCHAR; 256] = [0; 256];
    let mut item_data: LONG = 0;
    let mut display_list: HWND = GetDlgItem(hdlg, IDC_PS_DISPLAYLIST as ::core::ffi::c_int);
    let mut list: HWND = GetDlgItem(hdlg, src_list_id as ::core::ffi::c_int);

    old_pos = SendMessageW(display_list, LB_GETCURSEL as UINT, 0 as WPARAM, 0 as LPARAM) as LONG;
    if old_pos == -(1 as LONG) {
        old_pos = 0 as LONG;
    }

    SendMessageW(display_list, WM_SETREDRAW as UINT, 0 as WPARAM, 0 as LPARAM);
    SendMessageW(display_list, LB_RESETCONTENT as UINT, 0 as WPARAM, 0 as LPARAM);
    count = SendMessageW(list, LB_GETCOUNT as UINT, 0 as WPARAM, 0 as LPARAM) as LONG;
    i = 0 as LONG;
    while i < count {
        SendMessageW(list, LB_GETTEXT as UINT, i as WPARAM, &raw mut txt as *mut WCHAR as LPARAM);
        item_data = SendMessageW(list, LB_GETITEMDATA as UINT, i as WPARAM, 0 as LPARAM) as LONG;
        SendMessageW(display_list, LB_INSERTSTRING as UINT, i as WPARAM, &raw mut txt as *mut WCHAR as LPARAM);
        SendMessageW(display_list, LB_SETITEMDATA as UINT, i as WPARAM, item_data as LPARAM);
        i += 1;
    }
    SendMessageW(display_list, LB_SETCURSEL as UINT, 0 as WPARAM, 0 as LPARAM);
    SendMessageW(display_list, WM_SETREDRAW as UINT, 1 as WPARAM, 0 as LPARAM);
    if GetForegroundWindow() == hdlg {
        SetFocus(display_list);
    }
}

unsafe extern "C" fn init_lists(mut hdlg: HWND, mut ps_struct: *mut ps_struct_t) {
    let mut pastes_added: DWORD = init_pastelist(hdlg, (*ps_struct).ps);
    let mut links_added: DWORD = init_linklist(hdlg, (*ps_struct).ps);
    let mut check_id: UINT = 0;
    let mut list_id: UINT = 0;
    if (*ps_struct).flags & (PSF_SELECTPASTE | PSF_SELECTPASTELINK) as DWORD == 0 as DWORD {
        (*ps_struct).flags |= PSF_SELECTPASTE as DWORD;
    }
    if pastes_added == 0 && links_added == 0 {
        (*ps_struct).flags &= !(PSF_SELECTPASTE | PSF_SELECTPASTELINK) as DWORD;
    } else if pastes_added == 0 && (*ps_struct).flags & PSF_SELECTPASTE as DWORD != 0 {
        (*ps_struct).flags &= !PSF_SELECTPASTE as DWORD;
        (*ps_struct).flags |= PSF_SELECTPASTELINK as DWORD;
    } else if links_added == 0 && (*ps_struct).flags & PSF_SELECTPASTELINK as DWORD != 0 {
        (*ps_struct).flags &= !PSF_SELECTPASTELINK as DWORD;
        (*ps_struct).flags |= PSF_SELECTPASTE as DWORD;
    }
    check_id = 0 as UINT;
    list_id = 0 as UINT;
    if (*ps_struct).flags & PSF_SELECTPASTE as DWORD != 0 {
        check_id = IDC_PS_PASTE as UINT;
        list_id = IDC_PS_PASTELIST as UINT;
    } else if (*ps_struct).flags & PSF_SELECTPASTELINK as DWORD != 0 {
        check_id = IDC_PS_PASTELINK as UINT;
        list_id = IDC_PS_PASTELINKLIST as UINT;
    }
    CheckRadioButton(
        hdlg,
        (IDC_PS_PASTE as i32),
        (IDC_PS_PASTELINK as i32),
        check_id as ::core::ffi::c_int,
    );
    if list_id != 0 {
        update_display_list(hdlg, list_id);
    } else {
        EnableWindow(GetDlgItem(hdlg, (IDOK as i32)), 0 as BOOL);
    };
}

unsafe extern "C" fn update_src_text(hdlg: HWND, ps_struct: *const ps_struct_t) {
    let mut str: *mut WCHAR = core::ptr::null_mut();
    if ((*ps_struct).flags & (PSF_SELECTPASTE as DWORD)) != 0 {
        if !(*ps_struct).source_name.is_null() {
            str = (*ps_struct).source_name;
        } else {
            str = (*ps_struct).link_source_name;
        }
    } else if !(*ps_struct).link_source_name.is_null() {
        str = (*ps_struct).link_source_name;
    } else {
        str = (*ps_struct).source_name;
    }
    SetDlgItemTextW(hdlg, IDC_PS_SOURCETEXT as i32, str as LPCWSTR);
}

unsafe extern "C" fn update_as_icon(hdlg: HWND, ps_struct: *mut ps_struct_t) {
    let icon_display: HWND = GetDlgItem(hdlg, IDC_PS_ICONDISPLAY as _);
    let display_as_icon: HWND = GetDlgItem(hdlg, IDC_PS_DISPLAYASICON as _);
    let change_icon: HWND = GetDlgItem(hdlg, IDC_PS_CHANGEICON as _);

    /* FIXME. No as icon handling */
    (*ps_struct).flags &= !(PSF_CHECKDISPLAYASICON as DWORD);

    CheckDlgButton(
        hdlg,
        IDC_PS_DISPLAYASICON as _,
        ((*ps_struct).flags & (PSF_CHECKDISPLAYASICON as DWORD)) as UINT,
    );
    EnableWindow(display_as_icon, 0 as _);
    ShowWindow(icon_display, SW_HIDE as _);
    EnableWindow(icon_display, 0 as _);
    ShowWindow(change_icon, SW_HIDE as _);
    EnableWindow(change_icon, 0 as _);
}

unsafe extern "C" fn update_result_text(hdlg: HWND, ps_struct: *const ps_struct_t) {
    let mut resource_txt: [WCHAR; 200] = [0; 200];
    let mut res_id: UINT = 0;
    let mut pent: *mut OLEUIPASTEENTRYW = core::ptr::null_mut();
    let mut cur_sel: LONG = 0;
    static percent_s: [WCHAR; 3] = [0x0025u16, 0x0073, 0x0000];
    let mut result_txt: *mut WCHAR = core::ptr::null_mut();
    let mut ptr: *mut WCHAR = core::ptr::null_mut();

    cur_sel = SendMessageW(
        GetDlgItem(hdlg, (IDC_PS_DISPLAYLIST as i32)),
        LB_GETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as LONG;
    if cur_sel == -(1 as LONG) {
        return;
    }
    pent = SendMessageW(
        GetDlgItem(hdlg, (IDC_PS_DISPLAYLIST as i32)),
        LB_GETITEMDATA as UINT,
        cur_sel as WPARAM,
        0 as LPARAM,
    ) as *mut OLEUIPASTEENTRYW;

    if ((*ps_struct).flags & (PSF_SELECTPASTE as DWORD)) != 0 {
        if ((*ps_struct).flags & (PSF_CHECKDISPLAYASICON as DWORD)) != 0 {
            res_id = IDS_PS_PASTE_OBJECT_AS_ICON as UINT;
        } else {
            res_id = IDS_PS_PASTE_DATA as UINT;
        }
    } else {
        if ((*ps_struct).flags & (PSF_CHECKDISPLAYASICON as DWORD)) != 0 {
            res_id = IDS_PS_PASTE_LINK_OBJECT_AS_ICON as UINT;
        } else {
            res_id = IDS_PS_PASTE_LINK_DATA as UINT;
        }
    }

    LoadStringW(
        OLEDLG_hInstance,
        res_id,
        core::ptr::addr_of_mut!(resource_txt) as *mut WCHAR as LPWSTR,
        ARRAY_SIZE!(resource_txt) as c_int,
    );
    ptr = wcsstr(
        core::ptr::addr_of_mut!(resource_txt) as *mut WCHAR,
        core::ptr::addr_of!(percent_s) as *const WCHAR,
    ) as *mut WCHAR;
    if !ptr.is_null() {
        let mut result_txt_len: size_t = lstrlenW((*pent).lpstrResultText) as size_t;
        let mut offs: isize = (ptr as *mut c_char)
            .offset_from(core::ptr::addr_of_mut!(resource_txt) as *mut WCHAR as *mut c_char)
            as isize;
        result_txt = malloc(
            ((wcslen(core::ptr::addr_of_mut!(resource_txt) as *mut WCHAR) as size_t)
                .wrapping_add(result_txt_len)
                .wrapping_sub(1 as size_t))
                .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
        ) as *mut WCHAR;
        memcpy(
            result_txt as *mut c_void,
            core::ptr::addr_of_mut!(resource_txt) as *mut WCHAR as *const c_void,
            offs as size_t,
        );
        memcpy(
            (result_txt as *mut c_char).offset(offs as isize) as *mut c_void,
            (*pent).lpstrResultText as *const c_void,
            result_txt_len.wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
        );
        memcpy(
            (result_txt as *mut c_char)
                .offset(offs as isize)
                .offset((result_txt_len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as isize)
                as *mut c_void,
            ptr.offset(2 as isize) as *const c_void,
            ((lstrlenW(ptr.offset(2 as isize) as LPCWSTR)
                + 1 as c_int) as size_t)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
        );
    } else {
        result_txt = core::ptr::addr_of_mut!(resource_txt) as *mut WCHAR;
    }
    SetDlgItemTextW(hdlg, (IDC_PS_RESULTTEXT as i32), result_txt as LPCWSTR);

    if result_txt != core::ptr::addr_of_mut!(resource_txt) as *mut WCHAR {
        free(result_txt as *mut c_void);
    }
}

unsafe extern "C" fn selection_change(hdlg: HWND, ps_struct: *mut ps_struct_t) {
    update_as_icon(hdlg, ps_struct);
    update_result_text(hdlg, ps_struct);
}

unsafe extern "C" fn mode_change(mut hdlg: HWND, mut ps_struct: *mut ps_struct_t, mut id: UINT) {
    if id == IDC_PS_PASTE as UINT {
        (*ps_struct).flags &= !(PSF_SELECTPASTELINK as DWORD);
        (*ps_struct).flags |= PSF_SELECTPASTE as DWORD;
    } else {
        (*ps_struct).flags &= !(PSF_SELECTPASTE as DWORD);
        (*ps_struct).flags |= PSF_SELECTPASTELINK as DWORD;
    }

    update_src_text(hdlg, ps_struct);
    update_display_list(
        hdlg,
        (if id == IDC_PS_PASTE as UINT {
            IDC_PS_PASTELIST
        } else {
            IDC_PS_PASTELINKLIST
        }) as UINT,
    );
    selection_change(hdlg, ps_struct);
}

unsafe extern "C" fn post_help_msg(hdlg: HWND, ps_struct: *mut ps_struct_t) {
    PostMessageW((*(*ps_struct).ps).hWndOwner, oleui_msg_help, hdlg as WPARAM, IDD_PASTESPECIAL as LPARAM);
}

unsafe extern "C" fn send_end_dialog_msg(hdlg: HWND, ps_struct: *mut ps_struct_t, id: UINT) {
    SendMessageW(hdlg, oleui_msg_enddialog, id as WPARAM, 0 as LPARAM);
}

unsafe extern "C" fn update_structure(mut hdlg: HWND, mut ps_struct: *mut ps_struct_t) {
    let mut cur_sel: LONG = SendMessageW(
        GetDlgItem(hdlg, IDC_PS_DISPLAYLIST as core::ffi::c_int),
        LB_GETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as LONG;
    if cur_sel != (-(1)) as LONG {
        let mut pent: *mut OLEUIPASTEENTRYW = core::ptr::null_mut::<OLEUIPASTEENTRYW>();
        pent = SendMessageW(
            GetDlgItem(hdlg, IDC_PS_DISPLAYLIST as core::ffi::c_int),
            LB_GETITEMDATA as UINT,
            cur_sel as WPARAM,
            0 as LPARAM,
        ) as *mut OLEUIPASTEENTRYW;
        (*(*ps_struct).ps).nSelectedIndex = (*pent).dwScratchSpace as INT;
    }
    (*(*ps_struct).ps).dwFlags = (*ps_struct).flags;
    (*(*ps_struct).ps).fLink = (((*ps_struct).flags & (PSF_SELECTPASTELINK as DWORD))
        != (0 as DWORD)) as core::ffi::c_int as BOOL;
}

unsafe extern "C" fn free_structure(ps_struct: *mut ps_struct_t) {
    free((*ps_struct).type_name as *mut ::core::ffi::c_void);
    free((*ps_struct).source_name as *mut ::core::ffi::c_void);
    free((*ps_struct).link_type_name as *mut ::core::ffi::c_void);
    free((*ps_struct).link_source_name as *mut ::core::ffi::c_void);
    CoTaskMemFree((*ps_struct).app_name as LPVOID);
    free(ps_struct as *mut ::core::ffi::c_void);
}
unsafe extern "system" fn ps_dlg_proc(hdlg: HWND, msg: UINT, wp: WPARAM, lp: LPARAM) -> INT_PTR {
    /* native uses prop name "Structure", but we're not compatible
       with that so we'll prepend "Wine_". */
    static PROP_NAME: [u16; 15] = [
    0x0057u16, 0x0069, 0x006E, 0x0065, 0x005F, 0x0053, 0x0074, 0x0072, 0x0075, 0x0063,
    0x0074, 0x0075, 0x0072, 0x0065, 0x0000,
];
    let mut ps_struct: *mut ps_struct_t = core::ptr::null_mut();

    ps_struct = GetPropW(hdlg, &raw const PROP_NAME as LPCWSTR) as *mut ps_struct_t;

    if msg != WM_INITDIALOG {
        if ps_struct.is_null() {
            return 0;
        }

        if (*(*ps_struct).ps).lpfnHook.is_some() {
            let ret = (*(*ps_struct).ps)
                .lpfnHook
                .unwrap_unchecked()(hdlg, msg, wp, lp);
            if ret != 0 {
                return (ret) as INT_PTR;
            }
        }
    }

    match msg {
        WM_INITDIALOG => {
            ps_struct = malloc(core::mem::size_of::<ps_struct_t>()) as *mut ps_struct_t;
            (*ps_struct).ps = lp as *mut OLEUIPASTESPECIALW;
            (*ps_struct).type_name = core::ptr::null_mut();
            (*ps_struct).source_name = core::ptr::null_mut();
            (*ps_struct).link_type_name = core::ptr::null_mut();
            (*ps_struct).link_source_name = core::ptr::null_mut();
            (*ps_struct).app_name = core::ptr::null_mut();
            (*ps_struct).flags = (*(*ps_struct).ps).dwFlags;

            SetPropW(hdlg, &raw const PROP_NAME as LPCWSTR, ps_struct as *mut core::ffi::c_void);

            if ((*(*ps_struct).ps).dwFlags & PSF_SHOWHELP) == 0 {
                ShowWindow(GetDlgItem(hdlg, (IDC_OLEUIHELP as i32)), (SW_HIDE as i32));
                EnableWindow(GetDlgItem(hdlg, (IDC_OLEUIHELP as i32)), 0);
            }

            if !(*(*ps_struct).ps).lpszCaption.is_null() {
                SetWindowTextW(hdlg, (*(*ps_struct).ps).lpszCaption);
            }

            get_descriptors(hdlg, ps_struct);

            init_lists(hdlg, ps_struct);

            update_src_text(hdlg, ps_struct);

            selection_change(hdlg, ps_struct);

            SetFocus(GetDlgItem(hdlg, (IDC_PS_DISPLAYLIST as i32)));

            if (*(*ps_struct).ps).lpfnHook.is_some() {
                (*(*ps_struct).ps)
                    .lpfnHook
                    .unwrap_unchecked()(
                    hdlg,
                    msg,
                    0,
                    0,
                );
            }
            return (FALSE) as INT_PTR; /* use new focus */
        }
        WM_COMMAND => {
            match (LOWORD!(wp)) as u32 {
                IDC_PS_DISPLAYLIST => {
                    match (HIWORD!(wp)) as u32 {
                        LBN_SELCHANGE => {
                            selection_change(hdlg, ps_struct);
                            return (FALSE) as INT_PTR;
                        }
                        _ => {
                            return (FALSE) as INT_PTR;
                        }
                    }
                }
                IDC_PS_PASTE | IDC_PS_PASTELINK => {
                    match (HIWORD!(wp)) as u32 {
                        BN_CLICKED => {
                            mode_change(hdlg, ps_struct, (LOWORD!(wp)) as UINT);
                            return (FALSE) as INT_PTR;
                        }
                        _ => {
                            return (FALSE) as INT_PTR;
                        }
                    }
                }
                IDC_OLEUIHELP => {
                    match (HIWORD!(wp)) as u32 {
                        BN_CLICKED => {
                            post_help_msg(hdlg, ps_struct);
                            return FALSE as INT_PTR;
                        }
                        _ => {
                            return FALSE as INT_PTR;
                        }
                    }
                }
                IDOK | IDCANCEL => {
                    match (HIWORD!(wp)) as u32 {
                        BN_CLICKED => {
                            send_end_dialog_msg(hdlg, ps_struct, (LOWORD!(wp)) as UINT);
                            return FALSE as INT_PTR;
                        }
                        _ => {
                            return FALSE as INT_PTR;
                        }
                    }
                }
                _ => {}
            }
            return FALSE as INT_PTR;
        }
        _ => {
            if msg == oleui_msg_enddialog {
                if wp == (IDOK as WPARAM) {
                    update_structure(hdlg, ps_struct);
                }
                EndDialog(hdlg, wp as INT_PTR);
                /* native does its cleanup in WM_DESTROY */
                RemovePropW(hdlg, &raw const PROP_NAME as LPCWSTR);
                free_structure(ps_struct);
                return TRUE as INT_PTR;
            }
            return FALSE as INT_PTR;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn OleUIPasteSpecialA(psA: LPOLEUIPASTESPECIALA) -> UINT {
    let mut ps: OLEUIPASTESPECIALW = core::mem::zeroed();
    let mut ret: UINT;

    memcpy(
        core::ptr::addr_of_mut!(ps) as *mut core::ffi::c_void,
        psA as *const core::ffi::c_void,
        (*psA).cbStruct as usize,
    );

    ps.lpszCaption = strdupAtoW((*psA).lpszCaption as *const i8) as LPCWSTR;
    if !IS_INTRESOURCE!(ps.lpszTemplate) {
        ps.lpszTemplate = strdupAtoW((*psA).lpszTemplate as *const i8) as LPCWSTR;
    }

    if (*psA).cPasteEntries > 0 {
        let mut size: DWORD;
        let mut i: INT;
        let _elem_index = 0;
        size = ((*psA).cPasteEntries as usize)
            .wrapping_mul(core::mem::size_of::<tagOLEUIPASTEENTRYW>() as usize)
            as DWORD;

        ps.arrPasteEntries = malloc(size as usize) as LPOLEUIPASTEENTRYW;
        memcpy(
            ps.arrPasteEntries as *mut core::ffi::c_void,
            (*psA).arrPasteEntries as *const core::ffi::c_void,
            size as usize,
        );
        i = 0 as INT;
        while i < (*psA).cPasteEntries {
            (*ps.arrPasteEntries.offset(i as isize)).lpstrFormatName =
                strdupAtoW((*(*psA).arrPasteEntries.offset(i as isize)).lpstrFormatName as *const i8)
                    as LPCWSTR;
            (*ps.arrPasteEntries.offset(i as isize)).lpstrResultText =
                strdupAtoW((*(*psA).arrPasteEntries.offset(i as isize)).lpstrResultText as *const i8)
                    as LPCWSTR;
            i += 1;
        }
    }

    ret = OleUIPasteSpecialW(core::ptr::addr_of_mut!(ps));

    if (*psA).cPasteEntries > 0 {
        let mut i: INT;
        i = 0 as INT;
        while i < (*psA).cPasteEntries {
            free(
                (*ps.arrPasteEntries.offset(i as isize)).lpstrFormatName as *mut WCHAR
                    as *mut core::ffi::c_void,
            );
            free(
                (*ps.arrPasteEntries.offset(i as isize)).lpstrResultText as *mut WCHAR
                    as *mut core::ffi::c_void,
            );
            i += 1;
        }
        free(ps.arrPasteEntries as *mut core::ffi::c_void);
    }
    if !IS_INTRESOURCE!(ps.lpszTemplate) {
        free(ps.lpszTemplate as *mut WCHAR as *mut core::ffi::c_void);
    }
    free(ps.lpszCaption as *mut WCHAR as *mut core::ffi::c_void);

    /* Copy back the output fields */
    (*psA).dwFlags = ps.dwFlags;
    (*psA).lpSrcDataObj = ps.lpSrcDataObj;
    (*psA).nSelectedIndex = ps.nSelectedIndex;
    (*psA).fLink = ps.fLink;
    (*psA).hMetaPict = ps.hMetaPict;
    (*psA).sizel = ps.sizel;

    ret
}

#[no_mangle]
pub unsafe extern "system" fn OleUIPasteSpecialW(ps: LPOLEUIPASTESPECIALW) -> UINT {
    let mut dlg_templ: LPCDLGTEMPLATEW = (*ps).hResource as LPCDLGTEMPLATEW;
    let mut ret: UINT;

    if ((TRACE_ON!(__wine_dbch_ole)) != 0) {
        dump_pastespecial(ps);
    }

    if (*ps).lpSrcDataObj.is_null() {
        OleGetClipboard(core::ptr::addr_of_mut!((*ps).lpSrcDataObj));
    }

    if !(*ps).hInstance.is_null() || (*ps).hResource.is_null() {
        let hInst: HINSTANCE = if !(*ps).hInstance.is_null() {
            (*ps).hInstance
        } else {
            OLEDLG_hInstance
        };
        let name: *const WCHAR = if !(*ps).hInstance.is_null() {
            (*ps).lpszTemplate as *const WCHAR
        } else {
            MAKEINTRESOURCEW!(IDD_PASTESPECIAL4) as *const WCHAR
        };
        let mut hrsrc: HRSRC;

        if name.is_null() {
            return OLEUI_ERR_LPSZTEMPLATEINVALID;
        }
        hrsrc = FindResourceW(
            hInst as HMODULE,
            name as LPCWSTR,
            RT_DIALOG as LPWSTR as LPCWSTR,
        );
        if hrsrc.is_null() {
            return OLEUI_ERR_FINDTEMPLATEFAILURE;
        }
        dlg_templ = LoadResource(hInst, hrsrc) as LPCDLGTEMPLATEW;
        if dlg_templ.is_null() {
            return OLEUI_ERR_LOADTEMPLATEFAILURE;
        }
    }

    ret = DialogBoxIndirectParamW(
        OLEDLG_hInstance,
        dlg_templ as LPCDLGTEMPLATE,
        (*ps).hWndOwner,
        Some(ps_dlg_proc),
        ps as LPARAM,
    ) as UINT;

    return ret;
}
