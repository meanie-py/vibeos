use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn ros_dbg_log( cls: __wine_debug_class, ch: *mut __wine_debug_channel, file: *const core::ffi::c_char, func: *const core::ffi::c_char, line: core::ffi::c_int, fmt: *const core::ffi::c_char, ... );
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const SZOLEUI_MSG_HELPW: [WCHAR; 15] = [
    'O' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'E' as i32 as WCHAR,
    'U' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    '_' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'G' as i32 as WCHAR,
    '_' as i32 as WCHAR,
    'H' as i32 as WCHAR,
    'E' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

pub const SZOLEUI_MSG_ENDDIALOGW: [WCHAR; 20] = [
    'O' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'E' as i32 as WCHAR,
    'U' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    '_' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'G' as i32 as WCHAR,
    '_' as i32 as WCHAR,
    'E' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'A' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'O' as i32 as WCHAR,
    'G' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

#[no_mangle]
pub static mut OLEDLG_hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut cf_embed_source: UINT = 0;

#[no_mangle]
pub static mut cf_embedded_object: UINT = 0;

#[no_mangle]
pub static mut cf_link_source: UINT = 0;

#[no_mangle]
pub static mut cf_object_descriptor: UINT = 0;

#[no_mangle]
pub static mut cf_link_src_descriptor: UINT = 0;

#[no_mangle]
pub static mut cf_ownerlink: UINT = 0;

#[no_mangle]
pub static mut cf_filename: UINT = 0;

#[no_mangle]
pub static mut cf_filenamew: UINT = 0;

#[no_mangle]
pub static mut oleui_msg_help: UINT = 0;

#[no_mangle]
pub static mut oleui_msg_enddialog: UINT = 0;

unsafe extern "C" fn register_clipboard_formats() {
    static CF_EMBEDSOURCEW: [u16; 13] = [
    0x0045u16, 0x006D, 0x0062, 0x0065, 0x0064, 0x0020, 0x0053, 0x006F, 0x0075, 0x0072,
    0x0063, 0x0065, 0x0000,
];
    static CF_EMBEDDEDOBJECTW: [u16; 16] = [
    0x0045u16, 0x006D, 0x0062, 0x0065, 0x0064, 0x0064, 0x0065, 0x0064, 0x0020, 0x004F,
    0x0062, 0x006A, 0x0065, 0x0063, 0x0074, 0x0000,
];
    static CF_LINKSOURCEW: [u16; 12] = [
    0x004Cu16, 0x0069, 0x006E, 0x006B, 0x0020, 0x0053, 0x006F, 0x0075, 0x0072, 0x0063,
    0x0065, 0x0000,
];
    static CF_OBJECTDESCRIPTORW: [u16; 18] = [
    0x004Fu16, 0x0062, 0x006A, 0x0065, 0x0063, 0x0074, 0x0020, 0x0044, 0x0065, 0x0073,
    0x0063, 0x0072, 0x0069, 0x0070, 0x0074, 0x006F, 0x0072, 0x0000,
];
    static CF_LINKSRCDESCRIPTORW: [u16; 23] = [
    0x004Cu16, 0x0069, 0x006E, 0x006B, 0x0020, 0x0053, 0x006F, 0x0075, 0x0072, 0x0063,
    0x0065, 0x0020, 0x0044, 0x0065, 0x0073, 0x0063, 0x0072, 0x0069, 0x0070, 0x0074,
    0x006F, 0x0072, 0x0000,
];
    static CF_OWNERLINKW: [u16; 10] = [0x004Fu16, 0x0077, 0x006E, 0x0065, 0x0072, 0x004C, 0x0069, 0x006E, 0x006B, 0x0000];
    static CF_FILENAMEW: [u16; 9] = [0x0046u16, 0x0069, 0x006C, 0x0065, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
    static CF_FILENAMEWW: [u16; 10] = [0x0046u16, 0x0069, 0x006C, 0x0065, 0x004E, 0x0061, 0x006D, 0x0065, 0x0057, 0x0000];

    cf_object_descriptor = RegisterClipboardFormatW(CF_OBJECTDESCRIPTORW.as_ptr());
    cf_link_src_descriptor = RegisterClipboardFormatW(CF_LINKSRCDESCRIPTORW.as_ptr());
    cf_embed_source = RegisterClipboardFormatW(CF_EMBEDSOURCEW.as_ptr());
    cf_embedded_object = RegisterClipboardFormatW(CF_EMBEDDEDOBJECTW.as_ptr());
    cf_link_source = RegisterClipboardFormatW(CF_LINKSOURCEW.as_ptr());
    cf_ownerlink = RegisterClipboardFormatW(CF_OWNERLINKW.as_ptr());
    cf_filename = RegisterClipboardFormatW(CF_FILENAMEW.as_ptr());
    cf_filenamew = RegisterClipboardFormatW(CF_FILENAMEWW.as_ptr());
}

unsafe extern "C" fn register_messages() {
    oleui_msg_help = RegisterWindowMessageW((SZOLEUI_MSG_HELPW).as_ptr());
    oleui_msg_enddialog = RegisterWindowMessageW((SZOLEUI_MSG_ENDDIALOGW).as_ptr());
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, fdwReason: DWORD, fImpLoad: LPVOID) -> BOOL {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
            OLEDLG_hInstance = hinstDLL;
            register_clipboard_formats();
            register_messages();
        }
        _ => {}
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn OleUIAddVerbMenuA(
    object: *mut IOleObject,
    shorttype: LPCSTR,
    hMenu: HMENU,
    uPos: UINT,
    uIDVerbMin: UINT,
    uIDVerbMax: UINT,
    addConvert: BOOL,
    idConvert: UINT,
    lphMenu: *mut HMENU,
) -> BOOL {
    let mut shorttypeW: *mut WCHAR = core::ptr::null_mut();
    let mut ret: BOOL = 0;

    if !shorttype.is_null() {
        let mut len: INT = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            shorttype,
            -(1 as INT),
            core::ptr::null_mut(),
            0 as INT,
        ) as INT;
        shorttypeW = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as *mut WCHAR;
        if !shorttypeW.is_null() {
            MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                shorttype,
                -(1 as INT),
                shorttypeW as LPWSTR,
                len as INT,
            );
        }
    }

    ret = OleUIAddVerbMenuW(
        object,
        shorttypeW as LPCWSTR,
        hMenu,
        uPos,
        uIDVerbMin,
        uIDVerbMax,
        addConvert,
        idConvert,
        lphMenu,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        shorttypeW as PVOID,
    );
    return ret;
}

unsafe extern "C" fn is_verb_in_range(verb: *const OLEVERB, idmin: UINT, idmax: UINT) -> BOOL {
    if idmax == 0 {
        return (TRUE) as BOOL;
    }
    return ((((*verb).lVerb as UINT).wrapping_add(idmin)) <= idmax) as BOOL;
}

unsafe extern "C" fn get_next_insertable_verb(
    mut enumverbs: *mut IEnumOLEVERB,
    mut idmin: UINT,
    mut idmax: UINT,
    mut verb: *mut OLEVERB,
) -> HRESULT {
    memset(
        verb as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<OLEVERB>() as size_t,
    );
    while IEnumOLEVERB_Next(
        enumverbs,
        1 as ULONG,
        verb as LPOLEVERB,
        ::core::ptr::null_mut::<ULONG>(),
    ) == S_OK
    {
        if is_verb_in_range(verb, idmin, idmax) != 0
            && ((*verb).grfAttribs & OLEVERBATTRIB_ONCONTAINERMENU as DWORD) != 0
        {
            return S_OK;
        }

        CoTaskMemFree((*verb).lpszVerbName as LPVOID);
        memset(
            verb as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<OLEVERB>() as size_t,
        );
    }

    return S_FALSE;
}

unsafe extern "C" fn insert_verb_to_menu(menu: HMENU, idmin: UINT, verb: *const OLEVERB) {
    InsertMenuW(
        menu,
        !(0 as UINT),
        ((*verb).fuFlags | MF_BYPOSITION as UINT | MF_STRING as UINT) as UINT,
        ((*verb).lVerb as UINT_PTR).wrapping_add(idmin as UINT_PTR),
        (*verb).lpszVerbName as LPCWSTR,
    );
}

#[no_mangle]
pub unsafe extern "system" fn OleUIAddVerbMenuW(object: *mut IOleObject, shorttype: LPCWSTR, hMenu: HMENU, uPos: UINT, idmin: UINT, idmax: UINT, addConvert: BOOL, idConvert: UINT, ret_submenu: *mut HMENU) -> BOOL {
    let mut enumverbs: *mut IEnumOLEVERB = core::ptr::null_mut();
    let mut usertype: LPOLESTR = core::ptr::null_mut();
    let mut firstverb: OLEVERB = core::mem::zeroed();
    let mut verb: OLEVERB = core::mem::zeroed();
    let mut objecttype: *mut WCHAR;
    let mut resstrW: [WCHAR; 32] = [0; 32];
    let mut args: [DWORD_PTR; 2] = [0; 2];
    let mut singleverb: BOOL;
    let mut submenu: HMENU;
    let mut str: *mut WCHAR = core::ptr::null_mut();

    if !ret_submenu.is_null() {
        *ret_submenu = core::ptr::null_mut();
    }

    if hMenu.is_null() || ret_submenu.is_null() {
        return (FALSE) as BOOL;
    }

    /* check if we can get verbs at all */
    if !object.is_null() {
        IOleObject_EnumVerbs(object, core::ptr::addr_of_mut!(enumverbs));
    }

    LoadStringW(OLEDLG_hInstance, IDS_VERBMENU_OBJECT, core::ptr::addr_of_mut!(resstrW) as LPWSTR, (ARRAY_SIZE!(resstrW)) as i32);
    /* no object, or object without enumeration support */
    if object.is_null() || enumverbs.is_null() {
        RemoveMenu(hMenu, uPos, MF_BYPOSITION as UINT);
        InsertMenuW(hMenu, uPos, (MF_BYPOSITION | MF_STRING | MF_GRAYED) as UINT, idmin as UINT_PTR, core::ptr::addr_of_mut!(resstrW) as *mut WCHAR as LPCWSTR);
        return (FALSE) as BOOL;
    }

    /* root entry string */
    if shorttype.is_null() && (IOleObject_GetUserType(object, USERCLASSTYPE_SHORT as DWORD, core::ptr::addr_of_mut!(usertype)) == S_OK) {
        objecttype = usertype as *mut WCHAR;
    } else {
        objecttype = shorttype as *mut WCHAR;
    }

    /* iterate through verbs */

    /* find first suitable verb */
    get_next_insertable_verb(enumverbs, idmin, idmax, core::ptr::addr_of_mut!(firstverb));
    singleverb = (get_next_insertable_verb(enumverbs, idmin, idmax, core::ptr::addr_of_mut!(verb)) != S_OK) as BOOL;

    if singleverb != 0 && addConvert == 0 {
        LoadStringW(OLEDLG_hInstance, IDS_VERBMENU_SINGLEVERB_OBJECT, core::ptr::addr_of_mut!(resstrW) as LPWSTR, (ARRAY_SIZE!(resstrW)) as i32);

        args[0] = firstverb.lpszVerbName as DWORD_PTR;
        args[1] = objecttype as DWORD_PTR;

        FormatMessageW(FORMAT_MESSAGE_FROM_STRING | FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_ARGUMENT_ARRAY,
            core::ptr::addr_of_mut!(resstrW) as *mut WCHAR as LPCVOID, 0 as DWORD, 0 as DWORD, core::ptr::addr_of_mut!(str) as *mut WCHAR as LPWSTR, 0 as DWORD, core::ptr::addr_of_mut!(args) as *mut DWORD_PTR as *mut va_list);

        RemoveMenu(hMenu, uPos, MF_BYPOSITION as UINT);
        InsertMenuW(hMenu, uPos, (MF_BYPOSITION | MF_STRING) as UINT, idmin as UINT_PTR, str as LPCWSTR);
        CoTaskMemFree(firstverb.lpszVerbName as LPVOID);
        HeapFree(GetProcessHeap(), 0 as DWORD, str as PVOID);
        IEnumOLEVERB_Release(enumverbs);
        CoTaskMemFree(usertype as LPVOID);
        return (TRUE) as BOOL;
    }

    submenu = CreatePopupMenu();
    insert_verb_to_menu(submenu, idmin, core::ptr::addr_of_mut!(firstverb));
    CoTaskMemFree(firstverb.lpszVerbName as LPVOID);

    if singleverb == 0 {
        insert_verb_to_menu(submenu, idmin, core::ptr::addr_of_mut!(verb));
        CoTaskMemFree(verb.lpszVerbName as LPVOID);
    }

    while get_next_insertable_verb(enumverbs, idmin, idmax, core::ptr::addr_of_mut!(verb)) == S_OK {
        insert_verb_to_menu(submenu, idmin, core::ptr::addr_of_mut!(verb));
        CoTaskMemFree(verb.lpszVerbName as LPVOID);
    }

    /* convert verb is at the bottom of a popup, separated from verbs */
    if addConvert != 0 {
        LoadStringW(OLEDLG_hInstance, IDS_VERBMENU_CONVERT, core::ptr::addr_of_mut!(resstrW) as LPWSTR, (ARRAY_SIZE!(resstrW)) as i32);
        InsertMenuW(submenu, !(0 as UINT), (MF_BYPOSITION | MF_SEPARATOR) as UINT, 0 as UINT_PTR, core::ptr::null::<WCHAR>());
        InsertMenuW(submenu, !(0 as UINT), (MF_BYPOSITION | MF_STRING) as UINT, idConvert as UINT_PTR, core::ptr::addr_of_mut!(resstrW) as *mut WCHAR as LPCWSTR);
    }

    if !submenu.is_null() {
        *ret_submenu = submenu;
    }

    /* now submenu is ready, add root entry to original menu, attach submenu */
    LoadStringW(OLEDLG_hInstance, IDS_VERBMENU_OBJECT_WITH_NAME, core::ptr::addr_of_mut!(resstrW) as LPWSTR, (ARRAY_SIZE!(resstrW)) as i32);

    args[0] = objecttype as DWORD_PTR;
    FormatMessageW(FORMAT_MESSAGE_FROM_STRING | FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_ARGUMENT_ARRAY,
        core::ptr::addr_of_mut!(resstrW) as *mut WCHAR as LPCVOID, 0 as DWORD, 0 as DWORD, core::ptr::addr_of_mut!(str) as *mut WCHAR as LPWSTR, 0 as DWORD, core::ptr::addr_of_mut!(args) as *mut DWORD_PTR as *mut va_list);

    InsertMenuW(hMenu, uPos, (MF_BYPOSITION | MF_POPUP | MF_STRING) as UINT, submenu as UINT_PTR, str as LPCWSTR);
    HeapFree(GetProcessHeap(), 0 as DWORD, str as PVOID);
    IEnumOLEVERB_Release(enumverbs);
    CoTaskMemFree(usertype as LPVOID);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn OleUICanConvertOrActivateAs(
    rClsid: REFCLSID,
    fIsLinkedObject: BOOL,
    wFormat: WORD,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn OleUIInsertObjectW(lpOleUIInsertObject: LPOLEUIINSERTOBJECTW) -> UINT {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return OLEUI_FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn OleUIEditLinksA(lpOleUIEditLinks: LPOLEUIEDITLINKSA) -> UINT {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return OLEUI_FALSE as UINT;
}

#[no_mangle]
pub unsafe extern "system" fn OleUIEditLinksW(lpOleUIEditLinks: LPOLEUIEDITLINKSW) -> UINT {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return OLEUI_FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn OleUIChangeIconA(lpOleUIChangeIcon: LPOLEUICHANGEICONA) -> UINT {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return OLEUI_FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn OleUIChangeIconW(lpOleUIChangeIcon: LPOLEUICHANGEICONW) -> UINT {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return OLEUI_FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn OleUIConvertA(mut lpOleUIConvert: LPOLEUICONVERTA) -> UINT {
    FIXME!("(%p): stub\n", lpOleUIConvert);
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return OLEUI_FALSE as UINT;
}

#[no_mangle]
pub unsafe extern "system" fn OleUIConvertW(lpOleUIConvert: LPOLEUICONVERTW) -> UINT {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return OLEUI_FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn OleUIBusyA(mut lpOleUIBusy: LPOLEUIBUSYA) -> UINT {
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_FIXME as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oledlg/oledlg_main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"OleUIBusyA\0" as *const u8 as *const core::ffi::c_char,
            349 as core::ffi::c_int,
            b"(%p): stub\n\0" as *const u8 as *const core::ffi::c_char,
            lpOleUIBusy,
        );
    }
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return OLEUI_FALSE as UINT;
}

#[no_mangle]
pub unsafe extern "system" fn OleUIBusyW(lpOleUIBusy: LPOLEUIBUSYW) -> UINT {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return OLEUI_FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn OleUIUpdateLinksA(
    mut lpOleUILinkCntr: LPOLEUILINKCONTAINERA,
    mut hwndParent: HWND,
    mut lpszTitle: LPSTR,
    mut cLinks: INT,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn OleUIUpdateLinksW(
    mut lpOleUILinkCntr: LPOLEUILINKCONTAINERW,
    mut hwndParent: HWND,
    mut lpszTitle: LPWSTR,
    mut cLinks: INT,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "C" fn OleUIPromptUserA_v(mut nTemplate: INT, mut hwndParent: HWND, ap_list: *mut ::core::ffi::c_void) -> INT {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return (OLEUI_FALSE) as INT;
}



#[no_mangle]
pub unsafe extern "C" fn OleUIPromptUserW_v(mut nTemplate: INT, mut hwndParent: HWND, ap_list: *mut ::core::ffi::c_void) -> INT {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (OLEUI_FALSE) as INT;
}



#[no_mangle]
pub unsafe extern "system" fn OleUIObjectPropertiesA(
    lpOleUIObjectProps: LPOLEUIOBJECTPROPSA,
) -> UINT {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return OLEUI_FALSE as UINT;
}

#[no_mangle]
pub unsafe extern "system" fn OleUIObjectPropertiesW(
    lpOleUIObjectProps: LPOLEUIOBJECTPROPSW,
) -> UINT {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return OLEUI_FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn OleUIChangeSourceA(lpOleUIChangeSource: LPOLEUICHANGESOURCEA) -> UINT {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return OLEUI_FALSE as UINT;
}

#[no_mangle]
pub unsafe extern "system" fn OleUIChangeSourceW(lpOleUIChangeSource: LPOLEUICHANGESOURCEW) -> UINT {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return OLEUI_FALSE as UINT;
}
