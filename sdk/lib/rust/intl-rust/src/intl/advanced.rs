// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct CPStruct {
    pub CodePage: UINT,
    pub Flags: DWORD,
    pub Name: [WCHAR; MAX_PATH as usize],
    pub NextItem: *mut CPStruct,
}

pub type CPAGE = CPStruct;
pub type PCPAGE = *mut CPStruct;

pub const CODEPAGE_INSTALLED: u32 = 0x00000001;
pub const CODEPAGE_NOT_REMOVEABLE: u32 = 0x00000002;
pub const CODEPAGE_INSTALL: u32 = 0x00000004;
pub const CODEPAGE_REMOVE: u32 = 0x00000008;

pub static mut PCPage: PCPAGE = core::ptr::null_mut();
pub static mut bSpain: BOOL = 0;
pub static mut hLangList: HWND = core::ptr::null_mut();
unsafe extern "C" fn GetSupportedCP(
    hInf: HINF) -> BOOL {
    let mut szSection: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut Context: INFCONTEXT = core::mem::zeroed();
    let mut Context2: INFCONTEXT = core::mem::zeroed();
    let mut pCodePage: PCPAGE;
    let mut cpInfEx: CPINFOEX = core::mem::zeroed();
    let mut uiCodePage: UINT = 0;

    if SetupFindFirstLine(hInf,
                            w!("CodePages"),
                            core::ptr::null_mut(),
                            &mut Context) == 0
    {
        return (FALSE) as i32;
    }

    loop {
        if SetupGetIntField(&mut Context, 0, &mut uiCodePage as *mut UINT as PINT) != 0
        {
            pCodePage = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<CPAGE>()) as SIZE_T) as PCPAGE;
            if pCodePage.is_null() {
                return (FALSE) as i32;
            }

            (*pCodePage).CodePage = uiCodePage;
            (*pCodePage).Flags = 0;
            (*pCodePage).Name[0] = UNICODE_NULL;

            if GetCPInfoExW(uiCodePage, 0, &mut cpInfEx) != 0
            {
                wcscpy((*pCodePage).Name.as_mut_ptr(), cpInfEx.CodePageName.as_ptr());
            }
            else
            {
                SetupGetStringFieldW(&mut Context, 1, (*pCodePage).Name.as_mut_ptr(), MAX_PATH, core::ptr::null_mut());
            }

            if wcslen((*pCodePage).Name.as_ptr()) != 0
            {
                (*pCodePage).NextItem = PCPage;
                PCPage = pCodePage;

                wsprintf(szSection.as_mut_ptr(), w!("CODEPAGE_REMOVE_%d"), uiCodePage);

                if (uiCodePage == GetACP()) ||
                    (uiCodePage == GetOEMCP()) ||
                    (SetupFindFirstLineW(hInf, szSection.as_ptr(), w!("AddReg"), &mut Context2) == 0)
                {
                    (*pCodePage).Flags |= CODEPAGE_NOT_REMOVEABLE;
                }
            }
            else
            {
                HeapFree(GetProcessHeap(), 0, pCodePage as *mut core::ffi::c_void);
            }
        }

        if SetupFindNextLine(&mut Context, &mut Context) == 0 {
            break;
        }
    }

    return (TRUE) as i32;
}
unsafe extern "system" fn InstalledCPProc(lpStr: PWSTR) -> BOOL {
    let mut lpCP: PCPAGE;
    let uiCP: UINT;

    lpCP = PCPage;
    uiCP = (_wtol(lpStr)) as UINT;

    loop {
        if lpCP.is_null() {
            break;
        }

        if (*lpCP).CodePage == uiCP {
            (*lpCP).Flags |= CODEPAGE_INSTALLED;
            break;
        }

        lpCP = (*lpCP).NextItem;
    }

    TRUE as BOOL
}

unsafe extern "C" fn InitCodePagesList(hwndDlg: HWND) {
    let mut pCodePage: PCPAGE;
    let mut ItemIndex: INT;
    let hList: HWND;
    let mut column: LV_COLUMN = core::mem::zeroed();
    let mut item: LV_ITEM = core::mem::zeroed();
    let mut ListRect: RECT = core::mem::zeroed();
    let hIntlInf: HINF;

    hIntlInf = SetupOpenInfFileW(w!("intl.inf"), core::ptr::null_mut(), INF_STYLE_WIN4, core::ptr::null_mut());
    if hIntlInf == INVALID_HANDLE_VALUE {
        return;
    }

    if SetupOpenAppendInfFile(core::ptr::null_mut(), hIntlInf, core::ptr::null_mut()) == 0 {
        SetupCloseInfFile(hIntlInf);
        return;
    }

    if GetSupportedCP(hIntlInf) == 0 {
        return;
    }

    SetupCloseInfFile(hIntlInf);

    if EnumSystemCodePagesW(Some(InstalledCPProc), CP_INSTALLED) == 0 {
        /* Hack: EnumSystemCodePages returns FALSE on successful completion! */
        /* return; */
    }

    hList = GetDlgItem(hwndDlg, (IDC_CONV_TABLES) as i32);

    ZeroMemory!(&mut column as *mut _ as LPBYTE, core::mem::size_of::<LV_COLUMN>() as u32);
    column.mask = LVCF_FMT | LVCF_WIDTH;
    column.fmt = (LVCFMT_LEFT) as i32;
    GetClientRect(hList, &mut ListRect);
    column.cx = ListRect.right - GetSystemMetrics((SM_CYHSCROLL) as i32);
    ListView_InsertColumn!(hList, 0, &mut column);

    ListView_SetExtendedListViewStyle!(hList, LVS_EX_CHECKBOXES | LVS_EX_FULLROWSELECT);

    pCodePage = PCPage;

    loop {
        if pCodePage.is_null() {
            break;
        }

        ZeroMemory!(&mut item as *mut _ as LPBYTE, core::mem::size_of::<LV_ITEM>() as u32);
        item.mask = LVIF_TEXT | LVIF_PARAM | LVIF_STATE;
        item.state = 0;
        item.stateMask = LVIS_STATEIMAGEMASK;
        item.pszText = (*pCodePage).Name.as_mut_ptr();
        item.lParam = pCodePage as LPARAM;

        ItemIndex = ListView_InsertItem!(hList, &mut item);
        if ItemIndex != -1 {
            if ((*pCodePage).Flags & CODEPAGE_INSTALLED) != 0 {
                ListView_SetItemState!(hList, ItemIndex,
                                      INDEXTOSTATEIMAGEMASK!(LVIS_SELECTED),
                                      LVIS_STATEIMAGEMASK);
            } else {
                ListView_SetItemState!(hList, ItemIndex,
                                      INDEXTOSTATEIMAGEMASK!(LVIS_FOCUSED),
                                      LVIS_STATEIMAGEMASK);
            }
        }

        pCodePage = (*pCodePage).NextItem;
    }
}
unsafe extern "system" fn LocalesEnumProc(lpLocale: PWSTR) -> i32 {
    let mut lcid: LCID;
    let mut lang: [u16; 255] = [0; 255];
    let mut index: i32;
    let mut bNoShow: i32 = FALSE as BOOL;

    lcid = wcstoul(lpLocale, core::ptr::null_mut(), 16);

    if lcid == MAKELCID!(MAKELANGID!(LANG_SPANISH, SUBLANG_SPANISH), SORT_DEFAULT) ||
        lcid == MAKELCID!(MAKELANGID!(LANG_SPANISH, SUBLANG_SPANISH_MODERN), SORT_DEFAULT)
    {
        if bSpain == FALSE as BOOL
        {
            LoadStringW(hApplet, IDS_SPAIN, lang.as_mut_ptr(), 255);
            bSpain = TRUE as BOOL;
        }
        else
        {
            bNoShow = TRUE as BOOL;
        }
    }
    else
    {
        GetLocaleInfoW(lcid, LOCALE_SLANGUAGE, lang.as_mut_ptr(), ((core::mem::size_of_val(&lang) / core::mem::size_of::<u16>()) as u32) as c_int);
    }

    if bNoShow == FALSE as BOOL
    {
        index = SendMessageW(hLangList,
                             CB_ADDSTRING,
                             0,
                             lang.as_ptr() as LPARAM) as c_int;

        SendMessageW(hLangList,
                     CB_SETITEMDATA,
                     index as WPARAM,
                     lcid as LPARAM);
    }

    TRUE as BOOL
}

unsafe extern "C" fn InitLanguagesList(
    hwndDlg: HWND,
    pGlobalData: PGLOBALDATA,
) {
    let mut langSel: [u16; 255] = [0; 255];

    hLangList = GetDlgItem(hwndDlg, (IDC_LANGUAGE_COMBO) as i32);

    bSpain = (FALSE) as i32;
    EnumSystemLocalesW(Some(LocalesEnumProc), LCID_SUPPORTED);

    GetLocaleInfoW(
        (*pGlobalData).SystemLCID,
        LOCALE_SLANGUAGE,
        langSel.as_mut_ptr(),
        (core::mem::size_of_val(&langSel) / core::mem::size_of::<u16>()) as i32,
    );

    SendMessageW(
        hLangList,
        CB_SELECTSTRING,
        (-(1 as isize)) as WPARAM,
        langSel.as_ptr() as LPARAM,
    );
}
unsafe extern "C" fn GetCurrentDPI(szDPI: LPTSTR) {
    let mut dwType: u32;
    let mut dwSize: u32;
    let mut dwDPI: u32 = 0;
    let dwDefDPI: u32 = 0x00000060;
    let mut hKey: HKEY = core::ptr::null_mut();

    if RegCreateKeyExW(HKEY_LOCAL_MACHINE, w!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\FontDPI"), 0, core::ptr::null_mut(),
                        REG_OPTION_NON_VOLATILE, KEY_ALL_ACCESS, core::ptr::null_mut(), &mut hKey, core::ptr::null_mut()) != (ERROR_SUCCESS) as i32
    {
        wcscpy((szDPI).cast(), w!("96"));
        return;
    }

    dwType = REG_DWORD;
    dwSize = core::mem::size_of::<u32>() as u32;

    if RegQueryValueExW(hKey, w!("LogPixels"), core::ptr::null_mut(), &mut dwType, &mut dwDPI as *mut u32 as LPBYTE, &mut dwSize) != (ERROR_SUCCESS) as i32
    {
        if RegSetValueExW(hKey, w!("LogPixels"), 0, REG_DWORD, &dwDefDPI as *const u32 as LPBYTE, core::mem::size_of::<u32>() as u32) == (ERROR_SUCCESS) as i32
        {
            wcscpy((szDPI).cast(), w!("96"));
            RegCloseKey(hKey);
            return;
        }
    }
    else
    {
        wsprintf((szDPI).cast(), w!("%d"), dwDPI);
    }

    RegCloseKey(hKey);
}
unsafe extern "C" fn SaveFontSubstitutionSettings(
    hwnd: HWND,
    pGlobalData: PGLOBALDATA) {
    let mut szDefCP: [u16; 5 + 1] = [0; 5 + 1];
    let mut szSection: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut szDPI: [u16; 3 + 1] = [0; 3 + 1];
    let hFontInf: HINF;
    let Count: UINT;

    GetLocaleInfoW(MAKELCID!((*pGlobalData).SystemLCID, SORT_DEFAULT), LOCALE_IDEFAULTCODEPAGE, szDefCP.as_mut_ptr(), (core::mem::size_of_val(&szDefCP) / core::mem::size_of::<u16>()) as i32);
    GetCurrentDPI((szDPI.as_mut_ptr()).cast());

    wsprintf(szSection.as_mut_ptr(), w!("Font.CP%s.%s"), szDefCP.as_ptr(), szDPI.as_ptr());

    hFontInf = SetupOpenInfFileW(w!("font.inf"), core::ptr::null_mut(), INF_STYLE_WIN4, core::ptr::null_mut());
    if hFontInf == INVALID_HANDLE_VALUE
    {
        return;
    }

    if SetupOpenAppendInfFile(core::ptr::null_mut(), hFontInf, core::ptr::null_mut()) == 0
    {
        SetupCloseInfFile(hFontInf);
        return;
    }

    Count = SetupGetLineCount(hFontInf, szSection.as_ptr()) as UINT;
    if Count <= 0
    {
        return;
    }

    if SetupInstallFromInfSectionW(hwnd, hFontInf, szSection.as_ptr(), SPINST_REGISTRY & !SPINST_FILES,
                                     core::ptr::null_mut(), core::ptr::null_mut(), 0, None, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut()) == 0
    {
        PrintErrorMsgBox(IDS_ERROR_UNICODE);
    }

    SetupCloseInfFile(hFontInf);
}

unsafe extern "C" fn SaveFontLinkingSettings(hwnd: HWND, pGlobalData: PGLOBALDATA) {}
unsafe extern "C" fn SaveSystemSettings(
    pGlobalData: *mut GLOBALDATA) {
    let mut ACPPage: [u16; 9] = [0; 9];
    let mut OEMPage: [u16; 9] = [0; 9];
    let mut langKey: HKEY = core::ptr::null_mut();
    let mut ret: u32;
    let mut value: [u16; 5] = [0; 5];
    let mut valuesize: u32;

    ret = (GetLocaleInfoW(MAKELCID!((*pGlobalData).SystemLCID, SORT_DEFAULT), LOCALE_IDEFAULTCODEPAGE, OEMPage.as_mut_ptr(), (core::mem::size_of_val(&OEMPage) / core::mem::size_of::<u16>()) as i32)) as u32;
    if ret == 0
    {
        PrintErrorMsgBox(IDS_ERROR_OEM_CODE_PAGE);
        return;
    }

    ret = (GetLocaleInfoW(MAKELCID!((*pGlobalData).SystemLCID, SORT_DEFAULT), LOCALE_IDEFAULTANSICODEPAGE, ACPPage.as_mut_ptr(), (core::mem::size_of_val(&ACPPage) / core::mem::size_of::<u16>()) as i32)) as u32;
    if ret == 0
    {
        PrintErrorMsgBox(IDS_ERROR_ANSI_CODE_PAGE);
        return;
    }

    ret = (RegOpenKeyW(HKEY_LOCAL_MACHINE, w!("SYSTEM\\CurrentControlSet\\Control\\NLS\\CodePage"), &mut langKey)) as u32;
    if ret != ERROR_SUCCESS
    {
        PrintErrorMsgBox(IDS_ERROR_NLS_CODE_REG);
        return;
    }

    RegSetValueExW(langKey, w!("OEMCP"), 0, REG_SZ, OEMPage.as_ptr() as LPBYTE, ((wcslen(OEMPage.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD);
    RegSetValueExW(langKey, w!("ACP"), 0, REG_SZ, ACPPage.as_ptr() as LPBYTE, ((wcslen(ACPPage.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD);

    RegCloseKey(langKey);

    wsprintf(value.as_mut_ptr(), w!("%04hX"), LANGIDFROMLCID!((*pGlobalData).SystemLCID) as c_uint);
    valuesize = ((wcslen(value.as_ptr()) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32;

    ret = (RegOpenKeyW(HKEY_LOCAL_MACHINE, w!("SYSTEM\\CurrentControlSet\\Control\\NLS\\Language"), &mut langKey)) as u32;
    if ret != ERROR_SUCCESS
    {
        PrintErrorMsgBox(IDS_ERROR_NLS_KEY_REG);
        return;
    }

    RegSetValueExW(langKey, w!("Default"), 0, REG_SZ, value.as_ptr() as LPBYTE, valuesize);
    RegCloseKey(langKey);
}
#[no_mangle]
pub unsafe extern "C" fn ListViewCustomDraw(
    lParam: LPARAM) -> LRESULT {
    let lplvcd = lParam as LPNMLVCUSTOMDRAW;

    match (*lplvcd).nmcd.dwDrawStage {
        CDDS_PREPAINT => {
            return CDRF_NOTIFYITEMDRAW as INT_PTR;
        }

        CDDS_ITEMPREPAINT => {
            if (*((*lplvcd).nmcd.lItemlParam as PCPAGE)).Flags & CODEPAGE_NOT_REMOVEABLE != 0 {
                (*lplvcd).clrText = GetSysColor((COLOR_GRAYTEXT) as i32);
            } else {
                (*lplvcd).clrText = GetSysColor((COLOR_WINDOWTEXT) as i32);
            }
            (*lplvcd).clrTextBk = GetSysColor((COLOR_WINDOW) as i32);
            return CDRF_NEWFONT as LRESULT;
        }

        _ => {}
    }

    return CDRF_DODEFAULT as LRESULT;
}
#[no_mangle]
pub unsafe extern "system" fn AdvancedPageProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pGlobalData: PGLOBALDATA;

    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBALDATA;

    match uMsg {
        WM_INITDIALOG => {
            pGlobalData = ((*(lParam as LPPROPSHEETPAGE)).lParam) as PGLOBALDATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            InitLanguagesList(hwndDlg, pGlobalData);
            InitCodePagesList(hwndDlg);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_LANGUAGE_COMBO => {
                    if HIWORD!(wParam) == (CBN_SELCHANGE) as u16 {
                        let lcid: LCID;
                        let iIndex: i32;

                        iIndex = (SendMessage(hLangList, CB_GETCURSEL, 0, 0)) as i32;
                        if iIndex == CB_ERR {
                            return FALSE as INT_PTR;
                        }

                        lcid = SendMessage(hLangList, CB_GETITEMDATA, iIndex as WPARAM, 0) as LCID;
                        if lcid == CB_ERR as LCID {
                            return FALSE as INT_PTR;
                        }

                        (*pGlobalData).SystemLCID = lcid;

                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_APPLY_CUR_USER_DEF_PROFILE => {
                    if HIWORD!(wParam) == (BN_CLICKED) as u16 {
                        if SendDlgItemMessageW(hwndDlg, (IDC_APPLY_CUR_USER_DEF_PROFILE) as i32, BM_GETCHECK, 0, 0) != 0 {
                            ResourceMessageBox(
                                hwndDlg,
                                MB_OK | MB_ICONWARNING,
                                IDS_APPLY_DEFAULT_TITLE,
                                IDS_APPLY_DEFAULT_TEXT,
                            );
                            (*pGlobalData).bApplyToDefaultUser = (TRUE) as i32;
                        } else {
                            (*pGlobalData).bApplyToDefaultUser = (FALSE) as i32;
                        }

                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                _ => {}
            }
        }

        WM_NOTIFY => {
            if (*(lParam as LPNMHDR)).code == (PSN_APPLY) as u32 {
                PropSheet_UnChanged!(GetParent(hwndDlg), hwndDlg);

                SaveSystemSettings(pGlobalData);
                SaveFontSubstitutionSettings(hwndDlg, pGlobalData);
                SaveFontLinkingSettings(hwndDlg, pGlobalData);
            } else if (*(lParam as LPNMHDR)).idFrom == IDC_CONV_TABLES as UINT_PTR
                && (*(lParam as LPNMHDR)).code == NM_CUSTOMDRAW
            {
                SetWindowLongPtr(
                    hwndDlg,
                    DWLP_MSGRESULT,
                    ListViewCustomDraw(lParam) as LONG_PTR,
                );
                return TRUE as INT_PTR;
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "intl.h"
// #include <wingdi.h>
//
// #include <debug.h>
//
// typedef struct CPStruct
// {
//     UINT CodePage;
//     DWORD Flags;
//     WCHAR Name[MAX_PATH];
//     struct CPStruct *NextItem;
// } CPAGE, *PCPAGE;
//
// #define CODEPAGE_INSTALLED      0x00000001
// #define CODEPAGE_NOT_REMOVEABLE 0x00000002
// #define CODEPAGE_INSTALL        0x00000004
// #define CODEPAGE_REMOVE         0x00000008
//
// static PCPAGE PCPage = NULL;
// static BOOL bSpain = FALSE;
// static HWND hLangList;
//
// static BOOL
// GetSupportedCP(
//     HINF hInf)
// {
//     WCHAR szSection[MAX_PATH];
//     INFCONTEXT Context, Context2;
//     PCPAGE pCodePage;
//     CPINFOEX cpInfEx;
//     UINT uiCodePage;
//
//     if (!SetupFindFirstLine(hInf,
//                             L"CodePages",
//                             NULL,
//                             &Context))
//         return FALSE;
//
//     for (;;)
//     {
//         if (SetupGetIntField(&Context, 0, (PINT)&uiCodePage))
//         {
//             pCodePage = HeapAlloc(GetProcessHeap(), 0, sizeof(CPAGE));
//             if (pCodePage == NULL)
//                 return FALSE;
//
//             pCodePage->CodePage = uiCodePage;
//             pCodePage->Flags = 0;
//             (pCodePage->Name)[0] = UNICODE_NULL;
//
//             if (GetCPInfoExW(uiCodePage, 0, &cpInfEx))
//             {
//                 wcscpy(pCodePage->Name, cpInfEx.CodePageName);
//             }
//             else
//             {
//                 SetupGetStringFieldW(&Context, 1, pCodePage->Name, MAX_PATH, NULL);
//             }
//
//             if (wcslen(pCodePage->Name) != 0)
//             {
//                 pCodePage->NextItem = PCPage;
//                 PCPage = pCodePage;
//
//                 wsprintf(szSection, L"CODEPAGE_REMOVE_%d", uiCodePage);
//
//                 if ((uiCodePage == GetACP()) ||
//                     (uiCodePage == GetOEMCP()) ||
//                     (!SetupFindFirstLineW(hInf, szSection, L"AddReg", &Context2)))
//                 {
//                     pCodePage->Flags |= CODEPAGE_NOT_REMOVEABLE;
//                 }
//             }
//             else
//             {
//                 HeapFree(GetProcessHeap(), 0, pCodePage);
//             }
//         }
//
//         if (!SetupFindNextLine(&Context, &Context))
//             break;
//     }
//
//     return TRUE;
// }
//
// static BOOL __stdcall
// InstalledCPProc(PWSTR lpStr)
// {
//     PCPAGE lpCP;
//     UINT uiCP;
//
//     lpCP = PCPage;
//     uiCP = _wtol(lpStr);
//
//     for (;;)
//     {
//         if (!lpCP)
//             break;
//
//         if (lpCP->CodePage == uiCP)
//         {
//             lpCP->Flags |= CODEPAGE_INSTALLED;
//             break;
//         }
//
//         lpCP = lpCP->NextItem;
//     }
//
//     return TRUE;
// }
//
// static VOID
// InitCodePagesList(HWND hwndDlg)
// {
//     PCPAGE pCodePage;
//     INT ItemIndex;
//     HWND hList;
//     LV_COLUMN column;
//     LV_ITEM item;
//     RECT ListRect;
//     HINF hIntlInf;
//
//     hIntlInf = SetupOpenInfFileW(L"intl.inf", NULL, INF_STYLE_WIN4, NULL);
//     if (hIntlInf == INVALID_HANDLE_VALUE)
//         return;
//
//     if (!SetupOpenAppendInfFile(NULL, hIntlInf, NULL))
//     {
//         SetupCloseInfFile(hIntlInf);
//         hIntlInf = NULL;
//         return;
//     }
//
//     if (!GetSupportedCP(hIntlInf))
//         return;
//
//     SetupCloseInfFile(hIntlInf);
//
//     if (!EnumSystemCodePagesW(InstalledCPProc, CP_INSTALLED))
//     {
//         /* Hack: EnumSystemCodePages returns FALSE on successful completion! */
//         /* return; */
//     }
//
//     hList = GetDlgItem(hwndDlg, IDC_CONV_TABLES);
//
//     ZeroMemory(&column, sizeof(LV_COLUMN));
//     column.mask = LVCF_FMT | LVCF_WIDTH;
//     column.fmt = LVCFMT_LEFT;
//     GetClientRect(hList, &ListRect);
//     column.cx = ListRect.right - GetSystemMetrics(SM_CYHSCROLL);
//     ListView_InsertColumn(hList, 0, &column);
//
//     (VOID) ListView_SetExtendedListViewStyle(hList, LVS_EX_CHECKBOXES | LVS_EX_FULLROWSELECT);
//
//     pCodePage = PCPage;
//
//     for (;;)
//     {
//         if (pCodePage == NULL)
//             break;
//
//         ZeroMemory(&item, sizeof(LV_ITEM));
//         item.mask      = LVIF_TEXT | LVIF_PARAM | LVIF_STATE;
//         item.state     = 0;
//         item.stateMask = LVIS_STATEIMAGEMASK;
//         item.pszText   = pCodePage->Name;
//         item.lParam    = (LPARAM)pCodePage;
//
//         ItemIndex = ListView_InsertItem(hList, &item);
//         if (ItemIndex != -1)
//         {
//             if (pCodePage->Flags & CODEPAGE_INSTALLED)
//             {
//                 ListView_SetItemState(hList, ItemIndex,
//                                       INDEXTOSTATEIMAGEMASK(LVIS_SELECTED),
//                                       LVIS_STATEIMAGEMASK);
//             }
//             else
//             {
//                 ListView_SetItemState(hList, ItemIndex,
//                                       INDEXTOSTATEIMAGEMASK(LVIS_FOCUSED),
//                                       LVIS_STATEIMAGEMASK);
//             }
//         }
//
//         pCodePage = pCodePage->NextItem;
//     }
// }
//
// static BOOL __stdcall
// LocalesEnumProc(PWSTR lpLocale)
// {
//     LCID lcid;
//     WCHAR lang[255];
//     INT index;
//     BOOL bNoShow = FALSE;
//
//     lcid = wcstoul(lpLocale, NULL, 16);
//
//     if (lcid == MAKELCID(MAKELANGID(LANG_SPANISH, SUBLANG_SPANISH), SORT_DEFAULT) ||
//         lcid == MAKELCID(MAKELANGID(LANG_SPANISH, SUBLANG_SPANISH_MODERN), SORT_DEFAULT))
//     {
//         if (bSpain == FALSE)
//         {
//             LoadStringW(hApplet, IDS_SPAIN, lang, 255);
//             bSpain = TRUE;
//         }
//         else
//         {
//             bNoShow = TRUE;
//         }
//     }
//     else
//     {
//         GetLocaleInfoW(lcid, LOCALE_SLANGUAGE, lang, sizeof(lang)/sizeof(WCHAR));
//     }
//
//     if (bNoShow == FALSE)
//     {
//         index = SendMessageW(hLangList,
//                              CB_ADDSTRING,
//                              0,
//                              (LPARAM)lang);
//
//         SendMessageW(hLangList,
//                      CB_SETITEMDATA,
//                      index,
//                      (LPARAM)lcid);
//     }
//
//     return TRUE;
// }
//
// static VOID
// InitLanguagesList(
//     HWND hwndDlg,
//     PGLOBALDATA pGlobalData)
// {
//     WCHAR langSel[255];
//
//     hLangList = GetDlgItem(hwndDlg, IDC_LANGUAGE_COMBO);
//
//     bSpain = FALSE;
//     EnumSystemLocalesW(LocalesEnumProc, LCID_SUPPORTED);
//
//     /* Select current locale */
//     GetLocaleInfoW(pGlobalData->SystemLCID, LOCALE_SLANGUAGE, langSel, sizeof(langSel)/sizeof(WCHAR));
//
//     SendMessageW(hLangList, CB_SELECTSTRING, -1, (LPARAM)langSel);
// }
//
// static VOID
// GetCurrentDPI(LPTSTR szDPI)
// {
//     DWORD dwType, dwSize, dwDPI, dwDefDPI = 0x00000060; // Default 96 DPI
//     HKEY hKey;
//
//     if (RegCreateKeyExW(HKEY_LOCAL_MACHINE, L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\FontDPI", 0, NULL,
//                         REG_OPTION_NON_VOLATILE, KEY_ALL_ACCESS, NULL, &hKey, NULL) != ERROR_SUCCESS)
//     {
//         wcscpy(szDPI, L"96");
//         return;
//     }
//
//     dwType = REG_DWORD;
//     dwSize = sizeof(DWORD);
//
//     if (RegQueryValueExW(hKey, L"LogPixels", NULL, &dwType, (LPBYTE)&dwDPI, &dwSize) != ERROR_SUCCESS)
//     {
//         if (RegSetValueExW(hKey, L"LogPixels", 0, REG_DWORD, (LPBYTE)&dwDefDPI, sizeof(DWORD)) == ERROR_SUCCESS)
//         {
//             wcscpy(szDPI, L"96");
//             RegCloseKey(hKey);
//             return;
//         }
//     }
//     else
//     {
//         wsprintf(szDPI, L"%d", dwDPI);
//     }
//
//     RegCloseKey(hKey);
// }
//
// static
// VOID
// SaveFontSubstitutionSettings(
//     HWND hwnd,
//     PGLOBALDATA pGlobalData)
// {
//     WCHAR szDefCP[5 + 1], szSection[MAX_PATH], szDPI[3 + 1];
//     HINF hFontInf;
//     UINT Count;
//
//     GetLocaleInfoW(MAKELCID(pGlobalData->SystemLCID, SORT_DEFAULT), LOCALE_IDEFAULTCODEPAGE, szDefCP, sizeof(szDefCP) / sizeof(WCHAR));
//     GetCurrentDPI(szDPI);
//
//     wsprintf(szSection, L"Font.CP%s.%s", szDefCP, szDPI);
//
//     hFontInf = SetupOpenInfFileW(L"font.inf", NULL, INF_STYLE_WIN4, NULL);
//     if (hFontInf == INVALID_HANDLE_VALUE)
//         return;
//
//     if (!SetupOpenAppendInfFile(NULL, hFontInf, NULL))
//     {
//         SetupCloseInfFile(hFontInf);
//         return;
//     }
//
//     Count = (UINT)SetupGetLineCount(hFontInf, szSection);
//     if (Count <= 0)
//         return;
//
//     if (!SetupInstallFromInfSectionW(hwnd, hFontInf, szSection, SPINST_REGISTRY & ~SPINST_FILES,
//                                      NULL, NULL, 0, NULL, NULL, NULL, NULL))
//     {
//         PrintErrorMsgBox(IDS_ERROR_UNICODE);
//     }
//
//     SetupCloseInfFile(hFontInf);
// }
//
//
// static
// VOID
// SaveFontLinkingSettings(
//     HWND hwnd,
//     PGLOBALDATA pGlobalData)
// {
//     /* TODO */
// }
//
//
// static
// VOID
// SaveSystemSettings(
//     PGLOBALDATA pGlobalData)
// {
//     WCHAR ACPPage[9];
//     WCHAR OEMPage[9];
//     HKEY langKey;
//     DWORD ret;
//     WCHAR value[5];
//     DWORD valuesize;
//
//     ret = GetLocaleInfoW(MAKELCID(pGlobalData->SystemLCID, SORT_DEFAULT), LOCALE_IDEFAULTCODEPAGE, OEMPage, sizeof(OEMPage)/sizeof(WCHAR));
//     if (ret == 0)
//     {
//         PrintErrorMsgBox(IDS_ERROR_OEM_CODE_PAGE);
//         return;
//     }
//
//     ret = GetLocaleInfoW(MAKELCID(pGlobalData->SystemLCID, SORT_DEFAULT), LOCALE_IDEFAULTANSICODEPAGE, ACPPage, sizeof(ACPPage)/sizeof(WCHAR));
//     if (ret == 0)
//     {
//         PrintErrorMsgBox(IDS_ERROR_ANSI_CODE_PAGE);
//         return;
//     }
//
//     /* Set codepages */
//     ret = RegOpenKeyW(HKEY_LOCAL_MACHINE, L"SYSTEM\\CurrentControlSet\\Control\\NLS\\CodePage", &langKey);
//     if (ret != ERROR_SUCCESS)
//     {
//         PrintErrorMsgBox(IDS_ERROR_NLS_CODE_REG);
//         return;
//     }
//
//     RegSetValueExW(langKey, L"OEMCP", 0, REG_SZ, (BYTE *)OEMPage, (wcslen(OEMPage) +1 ) * sizeof(WCHAR));
//     RegSetValueExW(langKey, L"ACP", 0, REG_SZ, (BYTE *)ACPPage, (wcslen(ACPPage) +1 ) * sizeof(WCHAR));
//
//     RegCloseKey(langKey);
//
//
//     wsprintf(value, L"%04hX", LANGIDFROMLCID(pGlobalData->SystemLCID));
//     valuesize = (wcslen(value) + 1) * sizeof(WCHAR);
//
//     /* Set language */
//     ret = RegOpenKeyW(HKEY_LOCAL_MACHINE, L"SYSTEM\\CurrentControlSet\\Control\\NLS\\Language", &langKey);
//     if (ret != ERROR_SUCCESS)
//     {
//         PrintErrorMsgBox(IDS_ERROR_NLS_KEY_REG);
//         return;
//     }
//
//     RegSetValueExW(langKey, L"Default", 0, REG_SZ, (BYTE *)value, valuesize);
//     RegCloseKey(langKey);
// }
//
//
// LRESULT
// ListViewCustomDraw(
//     LPARAM lParam)
// {
//     LPNMLVCUSTOMDRAW lplvcd = (LPNMLVCUSTOMDRAW)lParam;
//
//     switch (lplvcd->nmcd.dwDrawStage)
//     {
//         case CDDS_PREPAINT:
//             return CDRF_NOTIFYITEMDRAW;
//
//         case CDDS_ITEMPREPAINT:
//             if (((PCPAGE)lplvcd->nmcd.lItemlParam)->Flags & CODEPAGE_NOT_REMOVEABLE)
//             {
//                 lplvcd->clrText = GetSysColor(COLOR_GRAYTEXT);
//             }
//             else
//             {
//                 lplvcd->clrText = GetSysColor(COLOR_WINDOWTEXT);
//             }
//             lplvcd->clrTextBk = GetSysColor(COLOR_WINDOW);
//             return CDRF_NEWFONT;
//     }
//
//     return CDRF_DODEFAULT;
// }
//
//
// /* Property page dialog callback */
// INT_PTR __stdcall
// AdvancedPageProc(HWND hwndDlg,
//                  UINT uMsg,
//                  WPARAM wParam,
//                  LPARAM lParam)
// {
//     PGLOBALDATA pGlobalData;
//
//     pGlobalData = (PGLOBALDATA)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             pGlobalData = (PGLOBALDATA)((LPPROPSHEETPAGE)lParam)->lParam;
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pGlobalData);
//
//             InitLanguagesList(hwndDlg, pGlobalData);
//             InitCodePagesList(hwndDlg);
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDC_LANGUAGE_COMBO:
//                     if (HIWORD(wParam) == CBN_SELCHANGE)
//                     {
//                         LCID lcid;
//                         INT iIndex;
//
//                         iIndex = SendMessage(hLangList, CB_GETCURSEL, 0, 0);
//                         if (iIndex == CB_ERR)
//                             break;
//
//                         lcid = SendMessage(hLangList, CB_GETITEMDATA, iIndex, 0);
//                         if (lcid == (LCID)CB_ERR)
//                             break;
//
//                         pGlobalData->SystemLCID = lcid;
//
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     }
//                     break;
//
//                 case IDC_APPLY_CUR_USER_DEF_PROFILE:
//                     if (HIWORD(wParam) == BN_CLICKED)
//                     {
//                         if (SendDlgItemMessageW(hwndDlg, IDC_APPLY_CUR_USER_DEF_PROFILE, BM_GETCHECK, 0, 0))
//                         {
//                             ResourceMessageBox(hwndDlg,
//                                                MB_OK | MB_ICONWARNING,
//                                                IDS_APPLY_DEFAULT_TITLE,
//                                                IDS_APPLY_DEFAULT_TEXT);
//                             pGlobalData->bApplyToDefaultUser = TRUE;
//                         }
//                         else
//                         {
//                             pGlobalData->bApplyToDefaultUser = FALSE;
//                         }
//
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     }
//                     break;
//             }
//             break;
//
//         case WM_NOTIFY:
//             if (((LPNMHDR)lParam)->code == PSN_APPLY)
//             {
//                 PropSheet_UnChanged(GetParent(hwndDlg), hwndDlg);
//
//                 SaveSystemSettings(pGlobalData);
//                 SaveFontSubstitutionSettings(hwndDlg, pGlobalData);
//                 SaveFontLinkingSettings(hwndDlg, pGlobalData);
//             }
//             else if (((LPNMHDR)lParam)->idFrom == IDC_CONV_TABLES &&
//                      ((LPNMHDR)lParam)->code == NM_CUSTOMDRAW)
//             {
//                 SetWindowLongPtr(hwndDlg,
//                                  DWLP_MSGRESULT,
//                                  (LONG_PTR)ListViewCustomDraw(lParam));
//                 return TRUE;
//             }
//             break;
//     }
//
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
