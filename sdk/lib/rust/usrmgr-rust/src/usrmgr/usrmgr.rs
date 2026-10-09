use crate::usrmgr::groups::GroupsPageProc;
use crate::usrmgr::users::UsersPageProc;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hApplet: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut Applets: [APPLET; 1] = unsafe {
    [_APPLET {
        idIcon: (IDI_USRMGR_ICON as i32),
        idName: (IDS_CPLNAME as i32),
        idDescription: (IDS_CPLDESCRIPTION as i32),
        AppletProc: Some(UsrmgrApplet),
    }]
};

pub const NUM_APPLETS: usize = 1;

// IDI_USRMGR_ICON, IDS_CPLNAME, IDS_CPLDESCRIPTION: values not given in the
// source; the numeric values below are placeholders and must be checked
// against the resource header.
#[no_mangle]
pub unsafe extern "C" fn GetDlgItemTextAlloc(hwndDlg: HWND, nDlgItem: INT) -> LPTSTR {
    let mut nLength: INT = GetWindowTextLength(GetDlgItem(hwndDlg, nDlgItem)) as INT;
    let mut psz: LPTSTR = HeapAlloc(
        GetProcessHeap(),
        0,
        ((nLength + 1) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<TCHAR>() as SIZE_T),
    ) as LPTSTR;
    if !psz.is_null() {
        GetDlgItemText(hwndDlg, nDlgItem, psz, nLength + 1);
    }
    return psz;
}
#[no_mangle]
pub unsafe extern "C" fn GetComboBoxLBTextAlloc(hwndDlg: HWND, nDlgItem: INT, nIndex: INT) -> LPTSTR {
    let nLength: INT = SendDlgItemMessage(
        hwndDlg,
        nDlgItem as ::core::ffi::c_int,
        CB_GETLBTEXTLEN as UINT,
        nIndex as WPARAM,
        0 as LPARAM,
    ) as INT;
    let psz: LPTSTR = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        ((nLength as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as SIZE_T)
            .wrapping_mul(::core::mem::size_of::<TCHAR>() as SIZE_T),
    ) as LPTSTR;
    if !psz.is_null() {
        SendDlgItemMessage(
            hwndDlg,
            nDlgItem as ::core::ffi::c_int,
            CB_GETLBTEXT as UINT,
            nIndex as WPARAM,
            psz as LPARAM,
        );
    }
    return psz;
}

unsafe extern "C" fn InitPropSheetPage(psp: *mut PROPSHEETPAGEW, idDlg: WORD, DlgProc: DLGPROC) {
    ZeroMemory!(psp as *mut core::ffi::c_void, core::mem::size_of::<PROPSHEETPAGE>());
    (*psp).dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    (*psp).dwFlags = PSP_DEFAULT as DWORD;
    (*psp).hInstance = hApplet;
    (*psp).__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(idDlg) as LPCWSTR;
    (*psp).pfnDlgProc = DlgProc;
}

unsafe extern "system" fn PropSheetProc(hwndDlg: HWND, uMsg: UINT, lParam: LPARAM) -> core::ffi::c_int {
    let mut hIcon: HICON = core::ptr::null_mut();
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW!(IDI_USRMGR_ICON as WORD) as *const u16);
            SendMessageW(hwndDlg, WM_SETICON as UINT, ICON_BIG as WPARAM, hIcon as LPARAM);
        }
        _ => {}
    }
    return 0;
}

unsafe extern "system" fn UsrmgrApplet(hwnd: HWND, uMsg: UINT, wParam: LPARAM, lParam: LPARAM) -> LONG {
    let mut psp: [PROPSHEETPAGE; 2] = core::mem::zeroed();
    let mut psh: PROPSHEETHEADER = core::mem::zeroed();

    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(wParam);
    UNREFERENCED_PARAMETER!(uMsg);

    ZeroMemory!(core::ptr::addr_of_mut!(psh) as *mut core::ffi::c_void, core::mem::size_of::<PROPSHEETHEADER>() as usize);
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADER>() as DWORD;
    psh.dwFlags = (PSH_PROPSHEETPAGE | PSH_USEICONID | PSH_USECALLBACK) as DWORD;
    psh.hwndParent = hwnd;
    psh.hInstance = hApplet;
    psh.__bindgen_anon_1.pszIcon = MAKEINTRESOURCEW!(IDI_USRMGR_ICON);
    psh.pszCaption = MAKEINTRESOURCEW!(IDS_CPLNAME);
    psh.nPages = (core::mem::size_of_val(&psp) / (core::mem::size_of::<PROPSHEETPAGE>() as usize)) as UINT;
    psh.__bindgen_anon_2.nStartPage = 0 as UINT;
    psh.__bindgen_anon_3.ppsp = core::ptr::addr_of_mut!(psp) as *mut PROPSHEETPAGE;
    psh.pfnCallback = Some(PropSheetProc);

    InitPropSheetPage(
        core::ptr::addr_of_mut!(psp[0 as usize]),
        IDD_USERS as WORD,
        Some(UsersPageProc),
    );
    InitPropSheetPage(
        core::ptr::addr_of_mut!(psp[1 as usize]),
        IDD_GROUPS as WORD,
        Some(GroupsPageProc),
    );
    /* InitPropSheetPage(&psp[2], IDD_EXTRA, ExtraPageProc); */

    return (PropertySheet(core::ptr::addr_of_mut!(psh))
        != -(1 as INT_PTR)) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn CPlApplet(
    hwndCPl: HWND,
    uMsg: UINT,
    lParam1: LPARAM,
    lParam2: LPARAM,
) -> LONG {
    let i: UINT = lParam1 as UINT;

    match uMsg {
        CPL_INIT => {
            return TRUE as LONG;
        }
        CPL_GETCOUNT => {
            return NUM_APPLETS as LONG;
        }
        CPL_INQUIRE => {
            if i < NUM_APPLETS as UINT {
                let CPlInfo: *mut CPLINFO = lParam2 as *mut CPLINFO;
                (*CPlInfo).lData = 0 as LONG_PTR;
                (*CPlInfo).idIcon = Applets[i as usize].idIcon;
                (*CPlInfo).idName = Applets[i as usize].idName;
                (*CPlInfo).idInfo = Applets[i as usize].idDescription;
            } else {
                return TRUE as LONG;
            }
        }
        CPL_DBLCLK => {
            if i < NUM_APPLETS as UINT {
                Applets[i as usize]
                    .AppletProc
                    .unwrap_unchecked()(
                    hwndCPl, uMsg, lParam1, lParam2
                );
            } else {
                return TRUE as LONG;
            }
        }
        _ => {}
    }

    return FALSE as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, dwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    UNREFERENCED_PARAMETER!(lpvReserved);

    match dwReason {
        DLL_PROCESS_ATTACH => {
            hApplet = hinstDLL;
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
