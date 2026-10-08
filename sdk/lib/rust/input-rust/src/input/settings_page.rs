use crate::input::input_list::InputList_Create;
use crate::input::input_list::InputList_Destroy;
use crate::input::input_list::InputList_GetAliveCount;
use crate::input::input_list::InputList_GetFirst;
use crate::input::input_list::InputList_Process;
use crate::input::input_list::InputList_Remove;
use crate::input::input_list::InputList_RemoveByLang;
use crate::input::input_list::InputList_SetDefault;
use crate::input::input_list::InputList_Sort;
use crate::input::locale_list::LocaleList_Create;
use crate::input::locale_list::LocaleList_Destroy;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_318DF2A56B41: [u16; 8] = [0x0025u16, 0x0073, 0x0020, 0x002D, 0x0020, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_BA2342D760A2: [u16; 6] = [0x0025u16, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut s_nAliveLeafCount: INT = 0 as INT;

static mut s_nRootCount: INT = 0 as INT;

static mut s_iKeyboardImage: INT = -(1 as INT);

static mut s_iDotImage: INT = -(1 as INT);

unsafe extern "C" fn CreateLayoutIcon(LangID: LANGID) -> HICON {
    let mut szBuf: [WCHAR; 4] = [0; 4];
    let mut hdcScreen: HDC = core::ptr::null_mut();
    let mut hdc: HDC = core::ptr::null_mut();
    let mut hbmColor: HBITMAP = core::ptr::null_mut();
    let mut hbmMono: HBITMAP = core::ptr::null_mut();
    let mut hBmpOld: HBITMAP = core::ptr::null_mut();
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut hFontOld: HFONT = core::ptr::null_mut();
    let mut lf: LOGFONTW = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();
    let mut IconInfo: ICONINFO = core::mem::zeroed();
    let mut hIcon: HICON = core::ptr::null_mut();
    let mut cxIcon: INT = GetSystemMetrics((SM_CXSMICON) as i32) as INT;
    let mut cyIcon: INT = GetSystemMetrics((SM_CYSMICON) as i32) as INT;

    /* Getting "EN", "FR", etc. from English, French, ... */
    if GetLocaleInfoW(
        LangID as LCID,
        (LOCALE_SABBREVLANGNAME | LOCALE_NOUSEROVERRIDE) as LCTYPE,
        core::ptr::addr_of_mut!(szBuf) as LPWSTR,
        ARRAYSIZE!(szBuf) as INT,
    ) == 0 as INT
    {
        szBuf[1 as usize] = 0x003Fu16;
        szBuf[0 as usize] = szBuf[1 as usize];
    }
    szBuf[2 as usize] = UNICODE_NULL as WCHAR; /* Truncate the identifier to two characters: "ENG" --> "EN" etc. */

    /* Create hdc, hbmColor and hbmMono */
    hdcScreen = GetDC(core::ptr::null_mut());
    hdc = CreateCompatibleDC(hdcScreen);
    hbmColor = CreateCompatibleBitmap(hdcScreen, cxIcon, cyIcon);
    ReleaseDC(core::ptr::null_mut(), hdcScreen);
    hbmMono = CreateBitmap(
        cxIcon,
        cyIcon,
        1 as UINT,
        1 as UINT,
        core::ptr::null(),
    );

    /* Checking NULL */
    if hdc.is_null() || hbmColor.is_null() || hbmMono.is_null() {
        if !hbmMono.is_null() {
            DeleteObject(hbmMono as HGDIOBJ);
        }
        if !hbmColor.is_null() {
            DeleteObject(hbmColor as HGDIOBJ);
        }
        if !hdc.is_null() {
            DeleteDC(hdc);
        }
        return core::ptr::null_mut();
    }

    /* Create a font */
    hFont = core::ptr::null_mut();
    if SystemParametersInfoW(
        SPI_GETICONTITLELOGFONT as UINT,
        core::mem::size_of::<LOGFONTW>() as UINT,
        core::ptr::addr_of_mut!(lf) as PVOID,
        0 as UINT,
    ) != 0
    {
        /* Override the current size with something manageable */
        lf.lfHeight = -(11 as LONG);
        lf.lfWidth = 0 as LONG;
        hFont = CreateFontIndirectW(core::ptr::addr_of_mut!(lf));
    }
    if hFont.is_null() {
        hFont = GetStockObject((DEFAULT_GUI_FONT as i32)) as HFONT;
    }

    SetRect(
        core::ptr::addr_of_mut!(rect),
        0 as INT,
        0 as INT,
        cxIcon,
        cyIcon,
    );

    /* Draw hbmColor */
    hBmpOld = SelectObject(hdc, hbmColor as HGDIOBJ) as HBITMAP;
    SetDCBrushColor(
        hdc,
        GetSysColor((COLOR_HIGHLIGHT) as i32) as COLORREF,
    );
    FillRect(
        hdc,
        core::ptr::addr_of_mut!(rect) as LPCRECT,
        GetStockObject((DC_BRUSH as i32)) as HBRUSH,
    );
    hFontOld = SelectObject(hdc, hFont as HGDIOBJ) as HFONT;
    SetTextColor(
        hdc,
        GetSysColor((COLOR_HIGHLIGHTTEXT) as i32) as COLORREF,
    );
    SetBkMode(hdc, TRANSPARENT as INT);
    DrawTextW(
        hdc,
        core::ptr::addr_of_mut!(szBuf) as LPCWSTR,
        2 as INT,
        core::ptr::addr_of_mut!(rect),
        (DT_SINGLELINE | DT_CENTER | DT_VCENTER) as UINT,
    );
    SelectObject(hdc, hFontOld as HGDIOBJ);

    /* Fill hbmMono with black */
    SelectObject(hdc, hbmMono as HGDIOBJ);
    PatBlt(
        hdc,
        0 as INT,
        0 as INT,
        cxIcon,
        cyIcon,
        BLACKNESS as DWORD,
    );
    SelectObject(hdc, hBmpOld as HGDIOBJ);

    /* Create an icon from hbmColor and hbmMono */
    IconInfo.fIcon = TRUE as BOOL;
    IconInfo.yHotspot = 0 as DWORD;
    IconInfo.xHotspot = IconInfo.yHotspot;
    IconInfo.hbmColor = hbmColor;
    IconInfo.hbmMask = hbmMono;
    hIcon = CreateIconIndirect(core::ptr::addr_of_mut!(IconInfo));

    /* Clean up */
    DeleteObject(hFont as HGDIOBJ);
    DeleteObject(hbmMono as HGDIOBJ);
    DeleteObject(hbmColor as HGDIOBJ);
    DeleteDC(hdc);

    return hIcon;
}

unsafe extern "C" fn InitDefaultLangComboBox(hwndCombo: HWND) {
    let mut szText: [WCHAR; 256] = [0; 256];
    let mut pNode: *mut INPUT_LIST_NODE = core::ptr::null_mut();
    let mut iIndex: INT;
    let mut nCount: INT;
    let mut iDefault: INT = SendMessageW(
        hwndCombo,
        CB_GETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as INT;

    SendMessageW(
        hwndCombo,
        CB_RESETCONTENT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );

    pNode = InputList_GetFirst();
    while !pNode.is_null() {
        'body: {
            if ((*pNode).wFlags & (INPUT_LIST_NODE_FLAG_DELETED as u16)) != 0 {
                break 'body;
            }

            StringCchPrintfW!(
                addr_of_mut!(szText) as *mut WCHAR,
                _countof!(szText),
                RUSTCONV_WSTR_318DF2A56B41.as_ptr(),
                (*(*pNode).pLocale).pszName,
                (*(*pNode).pLayout).pszName,
            );
            iIndex = SendMessageW(
                hwndCombo,
                CB_ADDSTRING as UINT,
                0 as WPARAM,
                addr_of_mut!(szText) as *mut WCHAR as LPARAM,
            ) as INT;
            SendMessageW(
                hwndCombo,
                CB_SETITEMDATA as UINT,
                iIndex as WPARAM,
                pNode as LPARAM,
            );

            if ((*pNode).wFlags & (INPUT_LIST_NODE_FLAG_DEFAULT as u16)) != 0 {
                iDefault = iIndex;
            }
        }
        pNode = (*pNode).pNext as *mut INPUT_LIST_NODE;
    }

    nCount = SendMessageW(
        hwndCombo,
        CB_GETCOUNT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as INT;
    if iDefault >= nCount {
        SendMessageW(
            hwndCombo,
            CB_SETCURSEL as UINT,
            (nCount - 1) as WPARAM,
            0 as LPARAM,
        );
    } else {
        SendMessageW(
            hwndCombo,
            CB_SETCURSEL as UINT,
            iDefault as WPARAM,
            0 as LPARAM,
        );
    }
}

unsafe extern "C" fn SetControlsState(hwndDlg: HWND) {
    let hwndList: HWND = GetDlgItem(hwndDlg, (IDC_KEYLAYOUT_LIST as i32));
    let hwndCombo: HWND = GetDlgItem(hwndDlg, (IDC_DEFAULT_LANGUAGE as i32));
    let mut bIsLeaf: BOOL;
    let mut bCanRemove: BOOL;
    let mut bCanProp: BOOL;
    let hSelected: HTREEITEM = TreeView_GetSelection!(hwndList);
    let mut item: TV_ITEM = core::mem::zeroed();
    item.mask = (TVIF_PARAM | TVIF_HANDLE) as UINT;
    item.hItem = hSelected;

    bIsLeaf = (!hSelected.is_null()
        && TreeView_GetItem!(hwndList, core::ptr::addr_of_mut!(item)) != 0
        && HIWORD!(item.lParam as DWORD) != 0) as BOOL;

    bCanRemove = ((bIsLeaf != 0 && s_nAliveLeafCount > 1)
        || s_nRootCount > 1) as BOOL;
    bCanProp = bIsLeaf;

    EnableWindow(GetDlgItem(hwndDlg, (IDC_REMOVE_BUTTON as i32)), bCanRemove);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_PROP_BUTTON as i32)), bCanProp);

    InitDefaultLangComboBox(hwndCombo);
}

unsafe extern "system" fn EnumResNameProc(
    mut hModule: HMODULE,
    mut lpszType: LPCWSTR,
    mut lpszName: LPWSTR,
    mut lParam: LONG_PTR,
) -> BOOL {
    let mut phIconSm: *mut HICON = lParam as *mut HICON;
    if !(*phIconSm).is_null() {
        return (FALSE) as BOOL;
    }
    *phIconSm = LoadImageW(
        hModule as HINSTANCE,
        lpszName as LPCWSTR,
        IMAGE_ICON as UINT,
        GetSystemMetrics((SM_CXSMICON) as i32),
        GetSystemMetrics((SM_CYSMICON) as i32),
        0 as UINT,
    ) as HICON;
    return (TRUE) as BOOL;
}

unsafe extern "C" fn LoadIMEIcon(pszImeFile: LPCTSTR) -> HICON {
    let mut szSysDir: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut hImeInst: HINSTANCE = core::ptr::null_mut();
    let mut hIconSm: HICON = core::ptr::null_mut();

    GetSystemDirectoryW(
        core::ptr::addr_of_mut!(szSysDir) as *mut WCHAR as LPWSTR,
        _countof!(szSysDir) as UINT,
    );
    StringCchPrintfW!(
        core::ptr::addr_of_mut!(szPath) as *mut WCHAR as STRSAFE_LPWSTR,
        _countof!(szPath) as size_t,
        RUSTCONV_WSTR_BA2342D760A2.as_ptr() as STRSAFE_LPCWSTR,
        core::ptr::addr_of_mut!(szSysDir) as *mut WCHAR,
        pszImeFile,
    );

    hImeInst = LoadLibraryExW(
        core::ptr::addr_of_mut!(szPath) as *mut WCHAR as LPCWSTR,
        core::ptr::null_mut(),
        DONT_RESOLVE_DLL_REFERENCES as DWORD,
    );
    if hImeInst.is_null() {
        return core::ptr::null_mut();
    }

    EnumResourceNamesW(
        hImeInst as HMODULE,
        RT_GROUP_ICON as LPCWSTR,
        Some(EnumResNameProc),
        core::ptr::addr_of_mut!(hIconSm) as LONG_PTR,
    );
    FreeLibrary(hImeInst as HMODULE);

    return hIconSm;
}

unsafe extern "C" fn FindLanguageInList(hwndList: HWND, pszLangName: LPCTSTR) -> HTREEITEM {
    let mut item: TVITEMW = core::mem::zeroed();
    let mut szText: [TCHAR; 128] = [0; 128];
    let mut hItem: HTREEITEM;

    hItem = TreeView_GetRoot!(hwndList);
    while !hItem.is_null() {
        szText[0 as usize] = 0 as TCHAR;
        item.mask = (TVIF_TEXT | TVIF_HANDLE) as UINT;
        item.pszText = &raw mut szText as *mut TCHAR as LPWSTR;
        item.cchTextMax = _countof!(szText) as ::core::ffi::c_int;
        item.hItem = hItem;
        TreeView_GetItem!(hwndList, &raw mut item);
        if _wcsicmp(&raw mut szText as *mut TCHAR, pszLangName) == 0 {
            return hItem;
        }

        hItem = TreeView_GetNextSibling!(hwndList, hItem);
    }

    return core::ptr::null_mut();
}

unsafe extern "C" fn AddToInputListView(hwndList: HWND, pInputNode: *mut INPUT_LIST_NODE) {
    let mut item: TVITEMW = core::mem::zeroed();
    let mut insert: TVINSERTSTRUCTW = core::mem::zeroed();
    let mut hImageList: HIMAGELIST = TreeView_GetImageList!(hwndList, TVSIL_NORMAL);
    let mut szKeyboard: [WCHAR; 64] = [0 as WCHAR; 64];
    let mut hItem: HTREEITEM = core::ptr::null_mut();
    let mut bBold: BOOL = ((((*pInputNode).wFlags as UINT) & (INPUT_LIST_NODE_FLAG_DEFAULT as UINT)) != 0) as BOOL;

    hItem = FindLanguageInList(hwndList, (*(*pInputNode).pLocale).pszName);
    if hItem.is_null() {
        // Language icon
        let mut LangImageIndex: INT = -1;
        let mut hLangIcon: HICON = CreateLayoutIcon(LOWORD!((*(*pInputNode).pLocale).dwId));
        if !hLangIcon.is_null() {
            LangImageIndex = ImageList_AddIcon!(hImageList, hLangIcon);
            DestroyIcon(hLangIcon);
        }

        // Language
        ZeroMemory!(&raw mut item as *mut core::ffi::c_void, core::mem::size_of::<TVITEMW>());
        item.mask = TVIF_TEXT | TVIF_IMAGE | TVIF_PARAM | TVIF_SELECTEDIMAGE | TVIF_STATE;
        item.pszText = (*(*pInputNode).pLocale).pszName;
        item.iImage = LangImageIndex;
        item.iSelectedImage = LangImageIndex;
        item.lParam = LOWORD!((*(*pInputNode).pLocale).dwId) as LPARAM; // HIWORD(item.lParam) == 0
        if bBold != 0 {
            item.stateMask = TVIS_BOLD;
            item.state = item.stateMask;
        }
        insert.hParent = TVI_ROOT;
        insert.hInsertAfter = TVI_LAST;
        insert.__bindgen_anon_1.item = item;
        hItem = TreeView_InsertItem!(hwndList, &raw mut insert);

        // The type of input method (currently keyboard only)
        LoadStringW(hApplet, IDS_KEYBOARD, &raw mut szKeyboard as *mut WCHAR, (_countof!(szKeyboard)) as INT);
        ZeroMemory!(&raw mut item as *mut core::ffi::c_void, core::mem::size_of::<TVITEMW>());
        item.mask = TVIF_TEXT | TVIF_IMAGE | TVIF_PARAM | TVIF_SELECTEDIMAGE;
        item.pszText = &raw mut szKeyboard as *mut WCHAR;
        item.iImage = s_iKeyboardImage;
        item.iSelectedImage = s_iKeyboardImage;
        item.lParam = 0; // HIWORD(item.lParam) == 0
        insert.hParent = hItem;
        insert.hInsertAfter = TVI_LAST;
        insert.__bindgen_anon_1.item = item;
        hItem = TreeView_InsertItem!(hwndList, &raw mut insert);
    } else {
        // Language
        ZeroMemory!(&raw mut item as *mut core::ffi::c_void, core::mem::size_of::<TVITEMW>());
        item.mask = TVIF_STATE | TVIF_HANDLE;
        item.hItem = hItem;
        item.stateMask = TVIS_BOLD;
        if TreeView_GetItem!(hwndList, &raw mut item) != 0
            && bBold != 0
            && (item.state & TVIS_BOLD) == 0
        {
            // Make the item bold
            item.mask = TVIF_STATE | TVIF_HANDLE;
            item.hItem = hItem;
            item.stateMask = TVIS_BOLD;
            item.state = item.stateMask;
            TreeView_SetItem!(hwndList, &raw mut item);
        }

        // The type of input method (currently keyboard only)
        hItem = TreeView_GetChild!(hwndList, hItem);
    }

    // Input method
    if !hItem.is_null() {
        let mut ImeImageIndex: INT = s_iDotImage;
        if IS_IME_HKL!((*pInputNode).hkl) != 0 && !(*(*pInputNode).pLayout).pszImeFile.is_null() // IME?
        {
            let mut hImeIcon: HICON = LoadIMEIcon((*(*pInputNode).pLayout).pszImeFile);
            if !hImeIcon.is_null() {
                ImeImageIndex = ImageList_AddIcon!(hImageList, hImeIcon);
                DestroyIcon(hImeIcon);
            }
        }

        ZeroMemory!(&raw mut item as *mut core::ffi::c_void, core::mem::size_of::<TVITEMW>());
        item.mask = TVIF_TEXT | TVIF_IMAGE | TVIF_PARAM | TVIF_SELECTEDIMAGE | TVIF_STATE;
        item.pszText = (*(*pInputNode).pLayout).pszName;
        item.iImage = ImeImageIndex;
        item.iSelectedImage = ImeImageIndex;
        item.lParam = pInputNode as LPARAM; // HIWORD(item.lParam) != 0
        if bBold != 0 {
            item.stateMask = TVIS_BOLD; // Make the item bold
            item.state = item.stateMask;
        }
        insert.hParent = hItem;
        insert.hInsertAfter = TVI_LAST;
        insert.__bindgen_anon_1.item = item;
        hItem = TreeView_InsertItem!(hwndList, &raw mut insert);
    }
}

unsafe extern "C" fn ExpandTreeItem(hwndTree: HWND, mut hItem: HTREEITEM) {
    TreeView_Expand!(hwndTree, hItem, TVE_EXPAND);
    hItem = TreeView_GetChild!(hwndTree, hItem);
    while !hItem.is_null() {
        ExpandTreeItem(hwndTree, hItem);
        hItem = TreeView_GetNextSibling!(hwndTree, hItem);
    }
}

unsafe extern "C" fn UpdateInputListView(hwndList: HWND) {
    let mut pNode: *mut INPUT_LIST_NODE = core::ptr::null_mut();
    let mut hImageList: HIMAGELIST = TreeView_GetImageList!(hwndList, TVSIL_NORMAL);
    let mut hItem: HTREEITEM = core::ptr::null_mut();
    let mut hKeyboardIcon: HICON = core::ptr::null_mut();
    let mut hDotIcon: HICON = core::ptr::null_mut();

    ImageList_RemoveAll!(hImageList);
    TreeView_DeleteAllItems!(hwndList);

    // Add keyboard icon
    s_iKeyboardImage = -1;
    hKeyboardIcon = LoadImageW(
        hApplet,
        MAKEINTRESOURCEW!(IDI_KEYBOARD) as LPCWSTR,
        IMAGE_ICON,
        GetSystemMetrics((SM_CXSMICON) as i32),
        GetSystemMetrics((SM_CYSMICON) as i32),
        0,
    ) as HICON;
    if !hKeyboardIcon.is_null() {
        s_iKeyboardImage = ImageList_AddIcon!(hImageList, hKeyboardIcon);
        DestroyIcon(hKeyboardIcon);
    }

    // Add dot icon
    s_iDotImage = -1;
    hDotIcon = LoadImageW(
        hApplet,
        MAKEINTRESOURCEW!(IDI_DOT) as LPCWSTR,
        IMAGE_ICON,
        GetSystemMetrics((SM_CXSMICON) as i32),
        GetSystemMetrics((SM_CYSMICON) as i32),
        0,
    ) as HICON;
    if !hDotIcon.is_null() {
        s_iDotImage = ImageList_AddIcon!(hImageList, hDotIcon);
        DestroyIcon(hDotIcon);
    }

    InputList_Sort();

    s_nAliveLeafCount = InputList_GetAliveCount();

    // Add items to the list
    pNode = InputList_GetFirst();
    while !pNode.is_null() {
        'body: {
            if ((*pNode).wFlags as i32 & (INPUT_LIST_NODE_FLAG_DELETED as i32)) != 0 {
                break 'body;
            }

            AddToInputListView(hwndList, pNode);
        }
        pNode = (*pNode).pNext as *mut INPUT_LIST_NODE;
    }

    // Expand all (with counting s_nRootCount)
    s_nRootCount = 0;
    hItem = TreeView_GetRoot!(hwndList);
    while !hItem.is_null() {
        s_nRootCount += 1;
        ExpandTreeItem(hwndList, hItem);
        hItem = TreeView_GetNextSibling!(hwndList, hItem);
    }

    // Redraw
    InvalidateRect(hwndList, core::ptr::null(), (TRUE) as BOOL);
}

unsafe extern "C" fn OnInitSettingsPage(hwndDlg: HWND) {
    let hwndInputList: HWND = GetDlgItem(hwndDlg, (IDC_KEYLAYOUT_LIST as i32));
    let mut hLayoutImageList: HIMAGELIST;
    let mut hOldImageList: HIMAGELIST;

    LayoutList_Create();
    LocaleList_Create();
    InputList_Create();

    EnableWindow(GetDlgItem(hwndDlg, (IDC_LANGUAGE_BAR as i32)), (FALSE) as BOOL);

    hLayoutImageList = ImageList_Create(GetSystemMetrics((SM_CXSMICON) as i32),
                                        GetSystemMetrics((SM_CYSMICON) as i32),
                                        ILC_COLOR8 | ILC_MASK, 0, 0);
    if !hLayoutImageList.is_null() {
        hOldImageList = TreeView_SetImageList!(hwndInputList, hLayoutImageList, TVSIL_NORMAL);
        ImageList_Destroy(hOldImageList);
    }

    UpdateInputListView(hwndInputList);

    SetControlsState(hwndDlg);
}

unsafe extern "C" fn OnDestroySettingsPage(hwndDlg: HWND) {
    LayoutList_Destroy();
    LocaleList_Destroy();
    InputList_Destroy();
}
#[no_mangle]
pub unsafe extern "C" fn OnCommandSettingsPage(hwndDlg: HWND, wParam: WPARAM) {
    match (LOWORD!(wParam)) as u32 {
        IDC_ADD_BUTTON => {
            if (DialogBoxW!(
                hApplet,
                MAKEINTRESOURCEW!(IDD_ADD),
                hwndDlg,
                Some(AddDialogProc),
            )) as u32 == (IDOK) as _
            {
                UpdateInputListView(GetDlgItem(hwndDlg, (IDC_KEYLAYOUT_LIST as i32)));
                SetControlsState(hwndDlg);
                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
            }
        }

        IDC_REMOVE_BUTTON => {
            let hwndList: HWND = GetDlgItem(hwndDlg, (IDC_KEYLAYOUT_LIST as i32));
            if !hwndList.is_null() {
                let hItem: HTREEITEM = TreeView_GetSelection!(hwndList);
                let mut item: TV_ITEM = core::mem::zeroed();
                item.mask = TVIF_HANDLE | TVIF_PARAM;
                item.hItem = hItem;

                if !hItem.is_null()
                    && TreeView_GetItem!(hwndList, core::ptr::addr_of_mut!(item)) != 0
                {
                    if item.lParam == 0 {
                        // Get root of branch
                        item.hItem = TreeView_GetParent!(hwndList, hItem);
                        TreeView_GetItem!(hwndList, core::ptr::addr_of_mut!(item));
                    }

                    if HIWORD!(item.lParam) != 0 {
                        // Leaf?
                        if InputList_Remove(item.lParam as *mut INPUT_LIST_NODE) != 0 {
                            g_bRebootNeeded = (TRUE) as i32;
                        }
                    } else {
                        // Root?
                        if InputList_RemoveByLang(LOWORD!(item.lParam)) != 0 {
                            g_bRebootNeeded = (TRUE) as i32;
                        }
                    }

                    UpdateInputListView(hwndList);
                    SetControlsState(hwndDlg);
                    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                }
            }
        }

        IDC_PROP_BUTTON => {
            let hwndList: HWND = GetDlgItem(hwndDlg, (IDC_KEYLAYOUT_LIST as i32));
            if !hwndList.is_null() {
                let hItem: HTREEITEM = TreeView_GetSelection!(hwndList);
                let mut item: TV_ITEM = core::mem::zeroed();
                item.mask = TVIF_HANDLE | TVIF_PARAM;
                item.hItem = hItem;

                if !hItem.is_null()
                    && TreeView_GetItem!(hwndList, core::ptr::addr_of_mut!(item)) != 0
                    && HIWORD!(item.lParam) != 0
                {
                    if (DialogBoxParamW(
                        hApplet,
                        MAKEINTRESOURCEW!(IDD_INPUT_LANG_PROP),
                        hwndDlg,
                        Some(EditDialogProc),
                        item.lParam,
                    )) as u32 == (IDOK) as _
                    {
                        UpdateInputListView(hwndList);
                        SetControlsState(hwndDlg);
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }
            }
        }

        IDC_KEY_SET_BTN => {
            DialogBoxW!(
                hApplet,
                MAKEINTRESOURCEW!(IDD_KEYSETTINGS),
                hwndDlg,
                Some(KeySettingsDialogProc),
            );
        }

        IDC_LANGUAGE_BAR => {
            // FIXME
        }

        IDC_DEFAULT_LANGUAGE => {
            if (HIWORD!(wParam)) as u32 == CBN_SELENDOK {
                let hwndList: HWND = GetDlgItem(hwndDlg, (IDC_KEYLAYOUT_LIST as i32));
                let hwndCombo: HWND = GetDlgItem(hwndDlg, (IDC_DEFAULT_LANGUAGE as i32));
                let iSelected: INT = SendMessageW(
                    hwndCombo,
                    CB_GETCURSEL,
                    0,
                    0,
                ) as INT;
                if iSelected != CB_ERR {
                    let lParam: LPARAM = SendMessageW(
                        hwndCombo,
                        CB_GETITEMDATA,
                        iSelected as WPARAM,
                        0,
                    );
                    if lParam != 0 {
                        let pNode: *mut INPUT_LIST_NODE = lParam as *mut INPUT_LIST_NODE;
                        if ((*pNode).wFlags & (INPUT_LIST_NODE_FLAG_DEFAULT as u16)) == 0 {
                            g_bRebootNeeded = (TRUE) as i32;
                            InputList_SetDefault(pNode);
                            UpdateInputListView(hwndList);
                            SetControlsState(hwndDlg);
                            PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                        }
                    }
                }
            }
        }

        _ => {}
    }
}
#[no_mangle]
pub unsafe extern "C" fn EnableProcessPrivileges(lpPrivilegeName: LPCWSTR, bEnable: BOOL) -> BOOL {
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut luid: LUID = core::mem::zeroed();
    let mut tokenPrivileges: TOKEN_PRIVILEGES = core::mem::zeroed();
    let mut Ret: BOOL;

    Ret = OpenProcessToken(
        GetCurrentProcess(),
        TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
        core::ptr::addr_of_mut!(hToken),
    );
    if Ret == 0 {
        return Ret;
    }

    Ret = LookupPrivilegeValueW(core::ptr::null(), lpPrivilegeName, core::ptr::addr_of_mut!(luid));
    if Ret != 0 {
        tokenPrivileges.PrivilegeCount = 1;
        (*core::ptr::addr_of_mut!(tokenPrivileges.Privileges))[0].Luid = luid;
        (*core::ptr::addr_of_mut!(tokenPrivileges.Privileges))[0].Attributes =
            if bEnable != 0 { SE_PRIVILEGE_ENABLED } else { 0 };

        Ret = AdjustTokenPrivileges(
            hToken,
            (FALSE) as BOOL,
            core::ptr::addr_of_mut!(tokenPrivileges),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    CloseHandle(hToken);
    return Ret;
}

unsafe extern "C" fn OnNotifySettingsPage(mut hwndDlg: HWND, mut lParam: LPARAM) -> INT_PTR {
    let mut header: LPNMHDR = lParam as LPNMHDR;
    // C `switch (header->code)` converts every label to the UINT scrutinee: the TVN_* are
    // `unsigned int` (TVN_FIRST is `0U-400U`), PSN_APPLY an `int`. Rust needs one type per
    // match, so the int label is converted as C converts it, in a guard.
    match (*header).code {
        TVN_SELCHANGED => {
            SetControlsState(hwndDlg);
        }
        TVN_ITEMEXPANDING => {
            // FIXME: Prevent collapse (COMCTL32 is buggy)
            // https://bugs.winehq.org/show_bug.cgi?id=53727
            let mut pTreeView: *mut NM_TREEVIEW = lParam as *mut NM_TREEVIEW;
            if ((*pTreeView).action & TVE_TOGGLE as UINT) == TVE_COLLAPSE as UINT {
                SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, TRUE as LONG_PTR);
                return TRUE as INT_PTR;
            }
        }
        code if code == PSN_APPLY as UINT => {
            /* Write Input Methods list to registry */
            g_bRebootNeeded |= InputList_Process();
        }
        _ => {}
    }
    return 0 as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn SettingsPageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            OnInitSettingsPage(hwndDlg);
            return TRUE as INT_PTR;
        }

        WM_DESTROY => {
            OnDestroySettingsPage(hwndDlg);
        }

        WM_COMMAND => {
            OnCommandSettingsPage(hwndDlg, wParam);
        }

        WM_NOTIFY => {
            return OnNotifySettingsPage(hwndDlg, lParam);
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
