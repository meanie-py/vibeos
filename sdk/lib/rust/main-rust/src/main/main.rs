use crate::main::keyboard::KeyboardApplet;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hApplet: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut Applets: [APPLET; 2] = unsafe {
    [
        _APPLET {
            idIcon: IDC_CPLICON_1 as UINT,
            idName: IDS_CPLNAME_1 as UINT,
            idDescription: IDS_CPLDESCRIPTION_1 as UINT,
            AppletProc: Some(
                MouseApplet,
            ),
        },
        _APPLET {
            idIcon: IDC_CPLICON_2 as UINT,
            idName: IDS_CPLNAME_2 as UINT,
            idDescription: IDS_CPLDESCRIPTION_2 as UINT,
            AppletProc: Some(
                KeyboardApplet,
            ),
        },
    ]
};

pub const NUM_APPLETS: usize = 2;

// IDC_CPLICON_*, IDS_CPLNAME_*, IDS_CPLDESCRIPTION_* are resource IDs from the
// applet's resource header; their values are not in this preamble.

// Field names for APPLET are assumed (struct definition not in this preamble).
#[no_mangle]
pub unsafe extern "C" fn InitPropSheetPage(
    mut ppsh: *mut PROPSHEETHEADERW,
    mut idDlg: WORD,
    mut DlgProc: DLGPROC,
) -> BOOL {
    let mut hPage: HPROPSHEETPAGE = core::ptr::null_mut();
    let mut psp: PROPSHEETPAGEW = core::mem::zeroed();

    if (*ppsh).nPages < (MAX_CPL_PAGES as u32) {
        ZeroMemory!(
            &raw mut psp as *mut core::ffi::c_void,
            core::mem::size_of::<PROPSHEETPAGEW>() as size_t,
        );
        psp.dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as DWORD;
        psp.dwFlags = PSP_DEFAULT;
        psp.hInstance = hApplet;
        psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(idDlg);
        psp.pfnDlgProc = DlgProc;

        hPage = CreatePropertySheetPage(&raw mut psp as LPCPROPSHEETPAGEW);
        if !hPage.is_null() {
            return PropSheetAddPage(hPage, ppsh as LPARAM);
        }
    }

    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PropSheetAddPage(hpage: HPROPSHEETPAGE, lParam: LPARAM) -> BOOL {
    let ppsh: *mut PROPSHEETHEADER = lParam as *mut PROPSHEETHEADER;
    if !ppsh.is_null() && (*ppsh).nPages < (MAX_CPL_PAGES as u32) {
        let idx = (*ppsh).nPages;
        (*ppsh).nPages = (*ppsh).nPages.wrapping_add(1);
        *(*ppsh).__bindgen_anon_3.phpage.offset(idx as isize) = hpage;
        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CPlApplet(
    mut hwndCpl: HWND,
    mut uMsg: UINT,
    mut lParam1: LPARAM,
    mut lParam2: LPARAM,
) -> LONG {
    let mut i: UINT = lParam1 as UINT;
    match uMsg {
        CPL_INIT => {
            return TRUE as LONG;
        }
        CPL_GETCOUNT => {
            return NUM_APPLETS as LONG;
        }
        CPL_INQUIRE => {
            if i < NUM_APPLETS as UINT {
                let mut CPlInfo: *mut CPLINFO = lParam2 as *mut CPLINFO;
                (*CPlInfo).lData = lParam1 as LONG_PTR;
                (*CPlInfo).idIcon = Applets[i as usize].idIcon as ::core::ffi::c_int;
                (*CPlInfo).idName = Applets[i as usize].idName as ::core::ffi::c_int;
                (*CPlInfo).idInfo = Applets[i as usize].idDescription as ::core::ffi::c_int;
            } else {
                return TRUE as LONG;
            }
        }
        CPL_DBLCLK => {
            if i < NUM_APPLETS as UINT {
                Applets[i as usize]
                    .AppletProc
                    .unwrap_unchecked()(
                    hwndCpl, uMsg, lParam1, lParam2
                );
            } else {
                return TRUE as LONG;
            }
        }
        CPL_STARTWPARMSW => {
            if i < NUM_APPLETS as UINT {
                return Applets[i as usize]
                    .AppletProc
                    .unwrap_unchecked()(
                    hwndCpl, uMsg, lParam1, lParam2
                );
            }
        }
        _ => {}
    }
    return FALSE as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: DWORD,
    lpReserved: LPVOID,
) -> BOOL {
    let mut InitControls: INITCOMMONCONTROLSEX = core::mem::zeroed();
    UNREFERENCED_PARAMETER!(lpReserved);

    match dwReason {
        DLL_PROCESS_ATTACH => {
            InitControls.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
            InitControls.dwICC = (ICC_LISTVIEW_CLASSES | ICC_UPDOWN_CLASS | ICC_BAR_CLASSES) as DWORD;
            InitCommonControlsEx(core::ptr::addr_of_mut!(InitControls));

            hApplet = hinstDLL;
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
