use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut OleUIInsertObjectInfoStr: [WCHAR; 25] = [
    'O' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'U' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'O' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    'j' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

#[repr(C)]
pub struct InsertObjectDlgInfo {
    pub hwndSelf: HWND,
    pub bObjListInit: BOOL, /* Object list has been initialized */
    pub lpOleUIInsertObject: LPOLEUIINSERTOBJECTA,

    pub hwndObjTypeLBL: HWND,
    pub hwndObjTypeLB: HWND,
    pub hwndFileLBL: HWND,
    pub hwndFileTB: HWND,
    pub hwndCreateCtrlCB: HWND,
    pub hwndCreateNewCB: HWND,
    pub hwndCreateFromFileCB: HWND,
    pub hwndDisplayIconCB: HWND,
    pub hwndAddCtrlBTN: HWND,
    pub hwndBrowseBTN: HWND,
    pub hwndResultDesc: HWND,
}

pub type DLLREGISTER = Option<unsafe extern "C" fn() -> HRESULT>;

#[no_mangle]

#[no_mangle]
pub unsafe extern "system" fn OleUIInsertObjectA(lpOleUIInsertObject: LPOLEUIINSERTOBJECTA) -> UINT {
    let mut lRes: LRESULT;
    let mut template: LPCVOID = core::ptr::null();
    let mut hRes: HRSRC = core::ptr::null_mut();
    let mut dlgInfo: InsertObjectDlgInfo = core::mem::zeroed();
    let mut hDlgTmpl: HANDLE = 0 as HANDLE;

    if !(*lpOleUIInsertObject).lpszTemplate.is_null() || !(*lpOleUIInsertObject).hResource.is_null() {
        FIXME!("Customized template not supported\n");
    }

    /* Create the dialog from a template */
    hRes = FindResourceA(
        OLEDLG_hInstance as HMODULE,
        MAKEINTRESOURCEA!(UIINSERTOBJECT),
        RT_DIALOG as LPSTR,
    );
    if hRes.is_null() {
        return OLEUI_ERR_FINDTEMPLATEFAILURE as UINT;
    }

    hDlgTmpl = LoadResource(OLEDLG_hInstance as HMODULE, hRes) as HANDLE;
    let mut bad: bool = hDlgTmpl.is_null();
    if !bad {
        template = LockResource(hDlgTmpl as HGLOBAL) as LPCVOID;
        bad = template.is_null();
    }
    if bad {
        return OLEUI_ERR_LOADTEMPLATEFAILURE as UINT;
    }

    /* Initialize InsertObjectDlgInfo structure */
    dlgInfo.lpOleUIInsertObject = lpOleUIInsertObject;
    dlgInfo.bObjListInit = FALSE as BOOL;

    lRes = DialogBoxIndirectParamA(
        OLEDLG_hInstance as HMODULE,
        template as *const DLGTEMPLATE,
        (*lpOleUIInsertObject).hWndOwner,
        Some(UIInsertObjectDlgProc),
        core::ptr::addr_of_mut!(dlgInfo) as LPARAM,
    ) as LRESULT;

    /* Unable to create the dialog */
    if lRes == (-(1 as LRESULT)) {
        return OLEUI_ERR_DIALOGFAILURE as UINT;
    }

    return lRes as UINT;
}
#[no_mangle]
pub unsafe extern "system" fn UIInsertObjectDlgProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pdlgInfo: *mut InsertObjectDlgInfo =
        GetPropW(hwnd, core::ptr::addr_of!(OleUIInsertObjectInfoStr) as LPCWSTR) as *mut InsertObjectDlgInfo;

    match uMsg {
        WM_INITDIALOG => {
            let pdlgInfo: *mut InsertObjectDlgInfo = lParam as *mut InsertObjectDlgInfo;

            (*pdlgInfo).hwndSelf = hwnd;

            SetPropW(
                hwnd,
                core::ptr::addr_of!(OleUIInsertObjectInfoStr) as LPCWSTR,
                pdlgInfo as HANDLE,
            );

            UIINSERTOBJECTDLG_InitDialog(pdlgInfo);

            return 0 as INT_PTR;
        }

        WM_COMMAND => return UIINSOBJDLG_OnWMCommand(hwnd, wParam, lParam) as INT_PTR,

        WM_DESTROY => {
            if !pdlgInfo.is_null() {
                UIINSERTOBJECTDLG_FreeObjectTypes(pdlgInfo);
            }
            RemovePropW(hwnd, core::ptr::addr_of!(OleUIInsertObjectInfoStr) as LPCWSTR);
            return FALSE as INT_PTR;
        }

        _ => return FALSE as INT_PTR,
    }
}

unsafe extern "C" fn UIINSOBJDLG_OnWMCommand(hwnd: HWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let pdlgInfo: *mut InsertObjectDlgInfo = GetPropW(hwnd, core::ptr::addr_of!(OleUIInsertObjectInfoStr) as LPCWSTR) as *mut InsertObjectDlgInfo;
    let wNotifyCode: WORD = HIWORD!(wParam);
    let wID: WORD = LOWORD!(wParam);

    match (wID as i32) as u32 {
        IDOK => {
            EndDialog(hwnd, UIINSERTOBJECTDLG_OnOpen(pdlgInfo) as INT_PTR);
        }
        IDCANCEL => {
            EndDialog(hwnd, FALSE as INT_PTR);
        }
        IDC_CREATECONTROL => {
            UIINSERTOBJECTDLG_SelectCreateCtrl(pdlgInfo);
        }
        IDC_CREATENEW => {
            UIINSERTOBJECTDLG_SelectCreateNew(pdlgInfo);
        }
        IDC_CREATEFROMFILE => {
            UIINSERTOBJECTDLG_SelectCreateFromFile(pdlgInfo);
        }
        IDC_BROWSE => {
            UIINSERTOBJECTDLG_BrowseFile(pdlgInfo);
        }
        IDC_ADDCONTROL => {
            UIINSERTOBJECTDLG_AddControl(pdlgInfo);
        }
        IDC_OBJTYPELIST => {
            if wNotifyCode as i32 == (LBN_SELCHANGE as i32) {
                UIINSERTOBJECTDLG_SelChange(pdlgInfo);
            }
        }
        _ => {}
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn UIINSERTOBJECTDLG_InitDialog(mut pdlgInfo: *mut InsertObjectDlgInfo) {
    (*pdlgInfo).hwndObjTypeLB = GetDlgItem((*pdlgInfo).hwndSelf, (IDC_OBJTYPELIST as i32));
    (*pdlgInfo).hwndObjTypeLBL = GetDlgItem((*pdlgInfo).hwndSelf, (IDC_OBJTYPELBL as i32));
    (*pdlgInfo).hwndFileLBL = GetDlgItem((*pdlgInfo).hwndSelf, (IDC_FILELBL as i32));
    (*pdlgInfo).hwndFileTB = GetDlgItem((*pdlgInfo).hwndSelf, (IDC_FILE as i32));
    (*pdlgInfo).hwndCreateCtrlCB = GetDlgItem((*pdlgInfo).hwndSelf, (IDC_CREATECONTROL as i32));
    (*pdlgInfo).hwndCreateNewCB = GetDlgItem((*pdlgInfo).hwndSelf, (IDC_CREATENEW as i32));
    (*pdlgInfo).hwndCreateFromFileCB = GetDlgItem((*pdlgInfo).hwndSelf, (IDC_CREATEFROMFILE as i32));
    (*pdlgInfo).hwndDisplayIconCB = GetDlgItem((*pdlgInfo).hwndSelf, (IDC_ASICON as i32));
    (*pdlgInfo).hwndAddCtrlBTN = GetDlgItem((*pdlgInfo).hwndSelf, (IDC_ADDCONTROL as i32));
    (*pdlgInfo).hwndBrowseBTN = GetDlgItem((*pdlgInfo).hwndSelf, (IDC_BROWSE as i32));
    (*pdlgInfo).hwndResultDesc = GetDlgItem((*pdlgInfo).hwndSelf, (IDC_RESULTDESC as i32));

    if !(*(*pdlgInfo).lpOleUIInsertObject).lpszCaption.is_null() {
        SetWindowTextA(
            (*pdlgInfo).hwndSelf,
            (*(*pdlgInfo).lpOleUIInsertObject).lpszCaption,
        );
    }

    ShowWindow(
        (*pdlgInfo).hwndCreateCtrlCB,
        if (*(*pdlgInfo).lpOleUIInsertObject).dwFlags & IOF_SHOWINSERTCONTROL as DWORD != 0 {
            (SW_SHOW as i32)
        } else {
            (SW_HIDE as i32)
        },
    );
    ShowWindow(
        (*pdlgInfo).hwndDisplayIconCB,
        if (*(*pdlgInfo).lpOleUIInsertObject).dwFlags & IOF_CHECKDISPLAYASICON as DWORD != 0 {
            (SW_SHOW as i32)
        } else {
            (SW_HIDE as i32)
        },
    );
    EnableWindow(
        (*pdlgInfo).hwndDisplayIconCB,
        ((*(*pdlgInfo).lpOleUIInsertObject).dwFlags & IOF_DISABLEDISPLAYASICON as DWORD == 0)
            as ::core::ffi::c_int,
    );

    if (*(*pdlgInfo).lpOleUIInsertObject).dwFlags & IOF_SELECTCREATECONTROL as DWORD != 0 {
        UIINSERTOBJECTDLG_SelectCreateCtrl(pdlgInfo);
    } else if (*(*pdlgInfo).lpOleUIInsertObject).dwFlags & IOF_SELECTCREATEFROMFILE as DWORD != 0 {
        UIINSERTOBJECTDLG_SelectCreateFromFile(pdlgInfo);
    } else {
        UIINSERTOBJECTDLG_SelectCreateNew(pdlgInfo);
    };
}

unsafe extern "C" fn UIINSERTOBJECTDLG_SelectCreateCtrl(mut pdlgInfo: *mut InsertObjectDlgInfo) {
    ShowWindow((*pdlgInfo).hwndDisplayIconCB, (SW_HIDE as i32));
    ShowWindow((*pdlgInfo).hwndFileLBL, (SW_HIDE as i32));
    ShowWindow((*pdlgInfo).hwndFileTB, (SW_HIDE as i32));
    ShowWindow((*pdlgInfo).hwndBrowseBTN, (SW_HIDE as i32));

    ShowWindow((*pdlgInfo).hwndObjTypeLBL, (SW_SHOW as i32));
    ShowWindow((*pdlgInfo).hwndObjTypeLB, (SW_SHOW as i32));
    ShowWindow((*pdlgInfo).hwndAddCtrlBTN, (SW_SHOW as i32));

    SendMessageA(
        (*pdlgInfo).hwndCreateCtrlCB,
        BM_SETCHECK as UINT,
        BST_CHECKED as WPARAM,
        0 as LPARAM,
    );

    /* Populate object type listbox */
    if (*pdlgInfo).bObjListInit == 0 {
        UIINSERTOBJECTDLG_PopulateObjectTypes(pdlgInfo);
    }
}

unsafe extern "C" fn UIINSERTOBJECTDLG_SelectCreateNew(pdlgInfo: *mut InsertObjectDlgInfo) {
    ShowWindow((*pdlgInfo).hwndFileLBL, (SW_HIDE as i32));
    ShowWindow((*pdlgInfo).hwndFileTB, (SW_HIDE as i32));
    ShowWindow((*pdlgInfo).hwndAddCtrlBTN, (SW_HIDE as i32));
    ShowWindow((*pdlgInfo).hwndBrowseBTN, (SW_HIDE as i32));

    if ((*(*pdlgInfo).lpOleUIInsertObject).dwFlags & (IOF_CHECKDISPLAYASICON as DWORD)) != 0 {
        ShowWindow((*pdlgInfo).hwndDisplayIconCB, (SW_SHOW as i32));
    }

    ShowWindow((*pdlgInfo).hwndObjTypeLBL, (SW_SHOW as i32));
    ShowWindow((*pdlgInfo).hwndObjTypeLB, (SW_SHOW as i32));

    SendMessageA(
        (*pdlgInfo).hwndCreateNewCB,
        BM_SETCHECK as UINT,
        BST_CHECKED as WPARAM,
        0 as LPARAM,
    );

    if (*pdlgInfo).bObjListInit == 0 {
        UIINSERTOBJECTDLG_PopulateObjectTypes(pdlgInfo);
    }

    UIINSERTOBJECTDLG_SelChange(pdlgInfo);
}

unsafe extern "C" fn UIINSERTOBJECTDLG_SelectCreateFromFile(mut pdlgInfo: *mut InsertObjectDlgInfo) {
    let mut resstr: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    ShowWindow((*pdlgInfo).hwndAddCtrlBTN, (SW_HIDE as i32));
    ShowWindow((*pdlgInfo).hwndObjTypeLBL, (SW_HIDE as i32));
    ShowWindow((*pdlgInfo).hwndObjTypeLB, (SW_HIDE as i32));

    if ((*(*pdlgInfo).lpOleUIInsertObject).dwFlags & IOF_CHECKDISPLAYASICON as DWORD) != 0 {
        ShowWindow((*pdlgInfo).hwndDisplayIconCB, (SW_SHOW as i32));
    }

    ShowWindow((*pdlgInfo).hwndFileLBL, (SW_SHOW as i32));
    ShowWindow((*pdlgInfo).hwndFileTB, (SW_SHOW as i32));
    ShowWindow((*pdlgInfo).hwndBrowseBTN, (SW_SHOW as i32));

    SendMessageW(
        (*pdlgInfo).hwndCreateFromFileCB,
        BM_SETCHECK as UINT,
        BST_CHECKED as WPARAM,
        0 as LPARAM,
    );

    if LoadStringW(
        OLEDLG_hInstance,
        IDS_RESULTFILEOBJDESC as UINT,
        &raw mut resstr as LPWSTR,
        (MAX_PATH as i32),
    ) != 0
    {
        SendMessageW(
            (*pdlgInfo).hwndResultDesc,
            WM_SETTEXT as UINT,
            0 as WPARAM,
            &raw mut resstr as *mut WCHAR as LPARAM,
        );
    }
}

unsafe extern "C" fn UIINSERTOBJECTDLG_PopulateObjectTypes(mut pdlgInfo: *mut InsertObjectDlgInfo) -> BOOL {
    static szClsid: [WCHAR; 6] = [0x0043u16, 0x004C, 0x0053, 0x0049, 0x0044, 0x0000];
    static szInsertable: [WCHAR; 11] = [
    0x0049u16, 0x006E, 0x0073, 0x0065, 0x0072, 0x0074, 0x0061, 0x0062, 0x006C, 0x0065,
    0x0000,
];
    static szNotInsertable: [WCHAR; 14] = [
    0x004Eu16, 0x006F, 0x0074, 0x0049, 0x006E, 0x0073, 0x0065, 0x0072, 0x0074, 0x0061,
    0x0062, 0x006C, 0x0065, 0x0000,
];
    let mut i: DWORD = 0;
    let mut len: LONG = 0;
    let mut hkclsids: HKEY = core::ptr::null_mut();
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut clsid: CLSID = core::mem::zeroed();
    let mut ret: LSTATUS = 0;
    let mut keydesc: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];
    let mut keyname: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];
    let mut szclsid: [WCHAR; 128] = [0 as WCHAR; 128];
    let mut index: DWORD = 0;

    UIINSERTOBJECTDLG_FreeObjectTypes(pdlgInfo);

    RegOpenKeyExW(
        HKEY_CLASSES_ROOT,
        &raw const szClsid as LPCWSTR,
        0,
        KEY_READ,
        &raw mut hkclsids,
    );

    loop {
        ret = RegEnumKeyW(
            hkclsids,
            index,
            &raw mut szclsid as LPWSTR,
            ARRAY_SIZE!(szclsid) as DWORD,
        );
        if !(ERROR_SUCCESS as LSTATUS == ret) {
            break;
        }

        index = index.wrapping_add(1);

        RegOpenKeyExW(
            hkclsids,
            &raw mut szclsid as *mut WCHAR as LPCWSTR,
            0,
            KEY_READ,
            &raw mut hkey,
        );

        len = core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as LONG;
        if ERROR_SUCCESS as LSTATUS
            != RegQueryValueW(
                hkey,
                &raw const szInsertable as LPCWSTR,
                &raw mut keyname as LPWSTR,
                &raw mut len,
            )
        {
            continue;
        }

        len = core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as LONG;
        if ERROR_SUCCESS as LSTATUS
            == RegQueryValueW(
                hkey,
                &raw const szNotInsertable as LPCWSTR,
                &raw mut keyname as LPWSTR,
                &raw mut len,
            )
        {
            continue;
        }

        CLSIDFromString(&raw mut szclsid as *mut WCHAR as LPCOLESTR, &raw mut clsid);

        i = 0;
        while i < (*(*pdlgInfo).lpOleUIInsertObject).cClsidExclude as DWORD {
            if IsEqualGUID!(
                (*(*pdlgInfo).lpOleUIInsertObject).lpClsidExclude.offset(i as isize) as *const GUID,
                &raw mut clsid as *const GUID,
            ) != false
            {
                break;
            }
            i = i.wrapping_add(1);
        }

        if i < (*(*pdlgInfo).lpOleUIInsertObject).cClsidExclude as DWORD {
            continue;
        }

        len = core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as LONG;
        if ERROR_SUCCESS as LSTATUS
            == RegQueryValueW(
                hkey,
                core::ptr::null(),
                &raw mut keydesc as LPWSTR,
                &raw mut len,
            )
        {
            let lpclsid: *mut CLSID = HeapAlloc(
                GetProcessHeap(),
                0 as DWORD,
                core::mem::size_of::<CLSID>() as SIZE_T,
            ) as *mut CLSID;
            *lpclsid = clsid;

            len = SendMessageW(
                (*pdlgInfo).hwndObjTypeLB,
                LB_ADDSTRING as UINT,
                0 as WPARAM,
                &raw mut keydesc as *mut WCHAR as LPARAM,
            ) as LONG;
            SendMessageW(
                (*pdlgInfo).hwndObjTypeLB,
                LB_SETITEMDATA as UINT,
                len as WPARAM,
                lpclsid as LPARAM,
            );
        }
    }

    (*pdlgInfo).bObjListInit =
        (ret == ERROR_NO_MORE_ITEMS as LSTATUS) as core::ffi::c_int as BOOL;

    return (*pdlgInfo).bObjListInit;
}

unsafe extern "C" fn UIINSERTOBJECTDLG_FreeObjectTypes(mut pdlgInfo: *mut InsertObjectDlgInfo) {
    let mut i: UINT = 0;
    let mut count: UINT = 0;
    count = SendMessageA(
        (*pdlgInfo).hwndObjTypeLB,
        LB_GETCOUNT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as UINT;
    i = 0 as UINT;
    while i < count {
        let mut lpclsid: *mut CLSID = SendMessageA(
            (*pdlgInfo).hwndObjTypeLB,
            LB_GETITEMDATA as UINT,
            i as WPARAM,
            0 as LPARAM,
        ) as *mut CLSID;
        HeapFree(GetProcessHeap(), 0 as DWORD, lpclsid as PVOID);
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn UIINSERTOBJECTDLG_SelChange(pdlgInfo: *mut InsertObjectDlgInfo) {
    let mut index: INT;
    let mut objname: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut objdesc: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut resstr: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if LoadStringW(
        OLEDLG_hInstance,
        IDS_RESULTOBJDESC as UINT,
        core::ptr::addr_of_mut!(resstr) as LPWSTR,
        MAX_PATH as INT,
    ) != 0
        && {
            index = SendMessageW(
                (*pdlgInfo).hwndObjTypeLB,
                LB_GETCURSEL as UINT,
                0 as WPARAM,
                0 as LPARAM,
            ) as INT;
            index >= 0 as INT
        }
        && SendMessageW(
            (*pdlgInfo).hwndObjTypeLB,
            LB_GETTEXT as UINT,
            index as WPARAM,
            core::ptr::addr_of_mut!(objname) as *mut WCHAR as LPARAM,
        ) != 0
    {
        wsprintfW(
            core::ptr::addr_of_mut!(objdesc) as LPWSTR,
            core::ptr::addr_of_mut!(resstr) as *mut WCHAR as LPCWSTR,
            core::ptr::addr_of_mut!(objname) as *mut WCHAR,
        );
    } else {
        objdesc[0 as usize] = 0 as WCHAR;
    }

    SendMessageW(
        (*pdlgInfo).hwndResultDesc,
        WM_SETTEXT as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(objdesc) as *mut WCHAR as LPARAM,
    );
}

unsafe extern "C" fn UIINSERTOBJECTDLG_OnOpen(pdlgInfo: *mut InsertObjectDlgInfo) -> BOOL {
    let mut bret: BOOL = FALSE as BOOL;

    if BST_CHECKED as LRESULT
        == SendMessageA(
            (*pdlgInfo).hwndCreateCtrlCB,
            BM_GETCHECK as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) as LRESULT
        || BST_CHECKED as LRESULT
            == SendMessageA(
                (*pdlgInfo).hwndCreateNewCB,
                BM_GETCHECK as UINT,
                0 as WPARAM,
                0 as LPARAM,
            ) as LRESULT
    {
        let mut index: INT = SendMessageA(
            (*pdlgInfo).hwndObjTypeLB,
            LB_GETCURSEL as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) as INT;

        if index >= 0 as INT {
            let mut clsid: *mut CLSID = SendMessageA(
                (*pdlgInfo).hwndObjTypeLB,
                LB_GETITEMDATA as UINT,
                index as WPARAM,
                0 as LPARAM,
            ) as *mut CLSID;
            (*(*pdlgInfo).lpOleUIInsertObject).clsid = *clsid;

            if (*(*pdlgInfo).lpOleUIInsertObject).dwFlags & IOF_CREATENEWOBJECT as DWORD != 0 {
                (*(*pdlgInfo).lpOleUIInsertObject).sc = OleCreate(
                    &raw mut (*(*pdlgInfo).lpOleUIInsertObject).clsid,
                    &raw mut (*(*pdlgInfo).lpOleUIInsertObject).iid,
                    (*(*pdlgInfo).lpOleUIInsertObject).oleRender,
                    (*(*pdlgInfo).lpOleUIInsertObject).lpFormatEtc,
                    (*(*pdlgInfo).lpOleUIInsertObject).lpIOleClientSite,
                    (*(*pdlgInfo).lpOleUIInsertObject).lpIStorage,
                    (*(*pdlgInfo).lpOleUIInsertObject).ppvObj,
                ) as SCODE;
            }

            bret = TRUE as BOOL;
        }
    } else if BST_CHECKED as LRESULT
        == SendMessageA(
            (*pdlgInfo).hwndCreateFromFileCB,
            BM_GETCHECK as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) as LRESULT
    {
        if !(*(*pdlgInfo).lpOleUIInsertObject).lpszFile.is_null() {
            let mut hres: HRESULT = 0;
            let mut wcsFile: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

            if SendMessageW(
                (*pdlgInfo).hwndFileTB,
                WM_GETTEXT as UINT,
                MAX_PATH as WPARAM,
                &raw mut wcsFile as *mut WCHAR as LPARAM,
            ) != 0
            {
                WideCharToMultiByte(
                    CP_ACP as UINT,
                    0 as DWORD,
                    &raw mut wcsFile as *mut WCHAR as LPCWSTR,
                    -(1 as INT),
                    (*(*pdlgInfo).lpOleUIInsertObject).lpszFile,
                    (*(*pdlgInfo).lpOleUIInsertObject).cchFile as INT,
                    ::core::ptr::null::<CHAR>(),
                    ::core::ptr::null_mut::<BOOL>(),
                );
            }

            hres = GetClassFile(
                &raw mut wcsFile as *mut WCHAR as LPCOLESTR,
                &raw mut (*(*pdlgInfo).lpOleUIInsertObject).clsid,
            );

            if hres >= 0 as HRESULT {
                if (*(*pdlgInfo).lpOleUIInsertObject).dwFlags & IOF_CREATEFILEOBJECT as DWORD != 0 {
                    hres = OleCreateFromFile(
                        &raw mut (*(*pdlgInfo).lpOleUIInsertObject).clsid,
                        &raw mut wcsFile as *mut WCHAR as LPCOLESTR,
                        &raw mut (*(*pdlgInfo).lpOleUIInsertObject).iid,
                        (*(*pdlgInfo).lpOleUIInsertObject).oleRender,
                        (*(*pdlgInfo).lpOleUIInsertObject).lpFormatEtc,
                        (*(*pdlgInfo).lpOleUIInsertObject).lpIOleClientSite,
                        (*(*pdlgInfo).lpOleUIInsertObject).lpIStorage,
                        (*(*pdlgInfo).lpOleUIInsertObject).ppvObj,
                    );
                }

                bret = TRUE as BOOL;
            }
            (*(*pdlgInfo).lpOleUIInsertObject).sc = hres as SCODE;
        }
    }

    return bret;
}

unsafe extern "C" fn UIINSERTOBJECTDLG_BrowseFile(pdlgInfo: *mut InsertObjectDlgInfo) {
    let mut fn_0: OPENFILENAMEA = core::mem::zeroed();
    let mut fname: [::core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut title: [::core::ffi::c_char; 32] = [0; 32];

    fn_0.lStructSize = core::mem::size_of::<OPENFILENAMEA>() as DWORD;
    fn_0.hwndOwner = (*pdlgInfo).hwndSelf;
    fn_0.hInstance = core::ptr::null_mut();
    fn_0.lpstrFilter = b"All Files\0*.*\0\0\0".as_ptr() as *const ::core::ffi::c_char as LPCSTR;
    fn_0.lpstrCustomFilter = core::ptr::null_mut();
    fn_0.nMaxCustFilter = 0 as DWORD;
    fn_0.nFilterIndex = 0 as DWORD;

    SendMessageA(
        (*pdlgInfo).hwndFileTB,
        WM_GETTEXT as UINT,
        MAX_PATH as WPARAM,
        core::ptr::addr_of_mut!(fname) as *mut ::core::ffi::c_char as LPARAM,
    );
    fn_0.lpstrFile = core::ptr::addr_of_mut!(fname) as *mut ::core::ffi::c_char as LPSTR;
    fn_0.nMaxFile = MAX_PATH as DWORD;

    fn_0.lpstrFileTitle = core::ptr::null_mut();
    fn_0.nMaxFileTitle = 0 as DWORD;
    fn_0.lpstrInitialDir = core::ptr::null();

    LoadStringA(
        OLEDLG_hInstance,
        IDS_BROWSE as UINT,
        core::ptr::addr_of_mut!(title) as *mut ::core::ffi::c_char as LPSTR,
        32 as ::core::ffi::c_int,
    );
    fn_0.lpstrTitle = core::ptr::addr_of_mut!(title) as *mut ::core::ffi::c_char as LPCSTR;

    fn_0.Flags = (OFN_EXPLORER | OFN_FILEMUSTEXIST
        | OFN_HIDEREADONLY | OFN_LONGNAMES) as DWORD;
    fn_0.nFileOffset = 0 as WORD;
    fn_0.nFileExtension = 0 as WORD;
    fn_0.lpstrDefExt = core::ptr::null();
    fn_0.lCustData = 0 as LPARAM;
    fn_0.lpfnHook = None;
    fn_0.lpTemplateName = core::ptr::null();

    if GetOpenFileNameA(core::ptr::addr_of_mut!(fn_0)) != 0 {
        SendMessageA(
            (*pdlgInfo).hwndFileTB,
            WM_SETTEXT as UINT,
            0 as WPARAM,
            fn_0.lpstrFile as LPARAM,
        );
    }
}

unsafe extern "C" fn UIINSERTOBJECTDLG_AddControl(pdlgInfo: *mut InsertObjectDlgInfo) {
    let mut fn_0: OPENFILENAMEA = core::mem::zeroed();
    let mut fname: [c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut title: [c_char; 32] = [0; 32];

    fn_0.lStructSize = core::mem::size_of::<OPENFILENAMEA>() as DWORD;
    fn_0.hwndOwner = (*pdlgInfo).hwndSelf;
    fn_0.hInstance = 0 as HINSTANCE;
    fn_0.lpstrFilter = b"OLE Controls\0*.ocx\0Libraries\0*.dll\0All Files\0*.*\0\0".as_ptr()
        as *const c_char as LPCSTR;
    fn_0.lpstrCustomFilter = core::ptr::null_mut::<CHAR>();
    fn_0.nMaxCustFilter = 0 as DWORD;
    fn_0.nFilterIndex = 0 as DWORD;

    fname[0 as usize] = 0 as c_char;
    fn_0.lpstrFile = core::ptr::addr_of_mut!(fname) as *mut c_char as LPSTR;
    fn_0.nMaxFile = MAX_PATH as DWORD;

    fn_0.lpstrFileTitle = core::ptr::null_mut::<CHAR>();
    fn_0.nMaxFileTitle = 0 as DWORD;
    fn_0.lpstrInitialDir = core::ptr::null::<CHAR>();

    LoadStringA(
        OLEDLG_hInstance,
        IDS_BROWSE as UINT,
        core::ptr::addr_of_mut!(title) as LPSTR,
        32 as c_int,
    );
    fn_0.lpstrTitle = core::ptr::addr_of_mut!(title) as *mut c_char as LPCSTR;

    fn_0.Flags = (OFN_EXPLORER | OFN_FILEMUSTEXIST
        | OFN_HIDEREADONLY | OFN_LONGNAMES) as DWORD;
    fn_0.nFileOffset = 0 as WORD;
    fn_0.nFileExtension = 0 as WORD;
    fn_0.lpstrDefExt = core::ptr::null::<CHAR>();
    fn_0.lCustData = 0 as LPARAM;
    fn_0.lpfnHook = None;
    fn_0.lpTemplateName = core::ptr::null::<CHAR>();

    if GetOpenFileNameA(core::ptr::addr_of_mut!(fn_0)) != 0 {
        let mut hMod: HMODULE = core::ptr::null_mut::<HINSTANCE__>();
        let mut bValid: BOOL = FALSE as BOOL;

        hMod = LoadLibraryA(fn_0.lpstrFile as LPCSTR) as HMODULE;

        if !hMod.is_null() {
            let mut regproc: DLLREGISTER = None;

            regproc = core::mem::transmute::<FARPROC, DLLREGISTER>(GetProcAddress(
                hMod as HINSTANCE,
                b"DllRegisterServer\0".as_ptr() as LPCSTR,
            ));
            if regproc.is_some() {
                let regproc = regproc.unwrap_unchecked();
                if S_OK as HRESULT == regproc() {
                    UIINSERTOBJECTDLG_PopulateObjectTypes(pdlgInfo);
                    bValid = TRUE as BOOL;
                }
            }

            FreeLibrary(hMod);
        }

        if bValid == 0 {
            let mut title: [WCHAR; 32] = [0; 32];
            let mut msg: [WCHAR; 256] = [0; 256];

            LoadStringW(
                OLEDLG_hInstance,
                IDS_NOTOLEMODCAPTION as UINT,
                core::ptr::addr_of_mut!(title) as LPWSTR,
                32 as c_int,
            );
            LoadStringW(
                OLEDLG_hInstance,
                IDS_NOTOLEMOD as UINT,
                core::ptr::addr_of_mut!(msg) as LPWSTR,
                256 as c_int,
            );

            MessageBoxW(
                (*pdlgInfo).hwndSelf,
                core::ptr::addr_of_mut!(msg) as *mut WCHAR as LPCWSTR,
                core::ptr::addr_of_mut!(title) as *mut WCHAR as LPCWSTR,
                MB_ICONEXCLAMATION as UINT,
            );
        }
    }
}
