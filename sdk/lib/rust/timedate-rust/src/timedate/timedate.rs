use crate::timedate::dateandtime::DateTimePageProc;
use crate::timedate::internettime::InetTimePageProc;
use crate::timedate::clock::RegisterClockControl;
use crate::timedate::monthcal::RegisterMonthCalControl;
use crate::timedate::clock::UnregisterClockControl;
use crate::timedate::monthcal::UnregisterMonthCalControl;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_155BFF67A742: [u16; 11] = [
    0x0025u16, 0x0068, 0x0073, 0x003A, 0x0025, 0x0064, 0x003A, 0x0020, 0x0025, 0x0073,
    0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hApplet: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut Applets: [APPLET; 1] = unsafe {
    [APPLET {
        idIcon: IDC_CPLICON as UINT,
        idName: IDS_CPLNAME as UINT,
        idDescription: IDS_CPLDESCRIPTION as UINT,
        AppletProc: Some(Applet),
    }]
};

pub const NUM_APPLETS: usize = 1;

// Assumed values: IDC_CPLICON / IDS_CPLNAME / IDS_CPLDESCRIPTION come from timedate resource.h (not in the given imports).
#[no_mangle]
pub unsafe extern "C" fn DisplayWin32ErrorDbg(dwErrorCode: DWORD, file: *const core::ffi::c_char, line: core::ffi::c_int) {
    let mut lpMsgBuf: PWSTR = core::ptr::null_mut();
    let mut szMsg: [WCHAR; 255] = [0; 255];

    FormatMessageW(
        FORMAT_MESSAGE_ALLOCATE_BUFFER |
        FORMAT_MESSAGE_FROM_SYSTEM |
        FORMAT_MESSAGE_IGNORE_INSERTS,
        core::ptr::null(),
        dwErrorCode,
        (MAKELANGID!(LANG_NEUTRAL, SUBLANG_DEFAULT)) as DWORD,
        &raw mut lpMsgBuf as LPWSTR,
        0,
        core::ptr::null_mut(),
    );

    if _swprintf(
        core::ptr::addr_of_mut!(szMsg) as *mut WCHAR,
        RUSTCONV_WSTR_155BFF67A742.as_ptr(),
        file,
        line,
        lpMsgBuf,
    ) != 0
    {
        MessageBoxW(
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(szMsg) as *mut WCHAR as LPCWSTR,
            core::ptr::null(),
            MB_OK | MB_ICONERROR,
        );
    }

    LocalFree(lpMsgBuf as HLOCAL);
}

unsafe extern "C" fn InitPropSheetPage(psp: *mut PROPSHEETPAGEW, idDlg: WORD, DlgProc: DLGPROC) {
    ZeroMemory!(
        psp as *mut core::ffi::c_void,
        core::mem::size_of::<PROPSHEETPAGEW>() as usize,
    );
    (*psp).dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as DWORD;
    (*psp).dwFlags = (PSP_DEFAULT) as DWORD;
    (*psp).hInstance = hApplet;
    (*psp).__bindgen_anon_1.pszTemplate = MAKEINTRESOURCEW!(idDlg) as LPCWSTR;
    (*psp).pfnDlgProc = DlgProc;
}

unsafe extern "system" fn PropSheetProc(hwndDlg: HWND, uMsg: UINT, lParam: LPARAM) -> i32 {
    let mut hIcon: HICON = core::ptr::null_mut();
    match uMsg {
        PSCB_INITIALIZED => {
            'block: {
                hIcon = LoadIconW(
                    hApplet,
                    MAKEINTRESOURCEW!(IDC_CPLICON) as LPCWSTR,
                ) as HICON;
                SendMessageW(
                    hwndDlg,
                    WM_SETICON as UINT,
                    ICON_BIG as WPARAM,
                    hIcon as LPARAM,
                );
                break 'block;
            }
        }
        _ => {}
    }
    return 0;
}
unsafe extern "system" fn Applet(mut hwnd: HWND, mut uMsg: UINT, mut wParam: LPARAM, mut lParam: LPARAM) -> LONG {
    let mut psh: PROPSHEETHEADERW = core::mem::zeroed();
    let mut psp: [PROPSHEETPAGEW; 3] = core::mem::zeroed();
    let mut Ret: LONG = 0 as LONG;

    let _ = uMsg;
    let _ = wParam;
    let _ = lParam;

    if RegisterMonthCalControl(hApplet) != 0 &&
        RegisterClockControl() != 0
    {
        ZeroMemory!(core::ptr::addr_of_mut!(psh) as *mut core::ffi::c_void, core::mem::size_of::<PROPSHEETHEADERW>());
        psh.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as DWORD;
        psh.dwFlags = PSH_PROPSHEETPAGE | PSH_PROPTITLE | PSH_USEICONID | PSH_USECALLBACK;
        psh.hwndParent = hwnd;
        psh.hInstance = hApplet;
        psh.__bindgen_anon_1.pszIcon = MAKEINTRESOURCEW!(IDC_CPLICON);
        psh.pszCaption = MAKEINTRESOURCEW!(IDS_CPLNAME);
        psh.nPages = (core::mem::size_of::<[PROPSHEETPAGEW; 3]>()
            / core::mem::size_of::<PROPSHEETPAGEW>()) as UINT;
        psh.__bindgen_anon_2.nStartPage = 0;
        psh.__bindgen_anon_3.ppsp = core::ptr::addr_of_mut!(psp) as *mut PROPSHEETPAGEW as *const PROPSHEETPAGEW;
        psh.pfnCallback = Some(PropSheetProc);

        InitPropSheetPage(core::ptr::addr_of_mut!(psp[0]), (IDD_DATETIMEPAGE) as WORD, Some(DateTimePageProc));
        InitPropSheetPage(core::ptr::addr_of_mut!(psp[1]), (IDD_TIMEZONEPAGE) as WORD, Some(TimeZonePageProc));
        InitPropSheetPage(core::ptr::addr_of_mut!(psp[2]), (IDD_INETTIMEPAGE) as WORD, Some(InetTimePageProc));

        Ret = (PropertySheetW(core::ptr::addr_of_mut!(psh)) != -1) as LONG;

        UnregisterMonthCalControl(hApplet);
        UnregisterClockControl();
    }

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn CPlApplet(
    hwndCpl: HWND,
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
            if i < (NUM_APPLETS) as UINT {
                let CPlInfo: *mut CPLINFO = lParam2 as *mut CPLINFO;
                (*CPlInfo).lData = 0 as LONG_PTR;
                (*CPlInfo).idIcon = Applets[i as usize].idIcon as _;
                (*CPlInfo).idName = Applets[i as usize].idName as _;
                (*CPlInfo).idInfo = Applets[i as usize].idDescription as _;
            } else {
                return TRUE as LONG;
            }
        }
        CPL_DBLCLK => {
            if i < (NUM_APPLETS) as UINT {
                Applets[i as usize]
                    .AppletProc
                    .unwrap_unchecked()(
                    hwndCpl, uMsg, lParam1, lParam2
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
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: DWORD,
    lpReserved: LPVOID,
) -> BOOL {
    UNREFERENCED_PARAMETER!(lpReserved);

    match dwReason {
        DLL_PROCESS_ATTACH => {
            let mut InitControls: INITCOMMONCONTROLSEX = core::mem::zeroed();

            InitControls.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
            InitControls.dwICC = (ICC_DATE_CLASSES | ICC_PROGRESS_CLASS | ICC_UPDOWN_CLASS) as DWORD;
            InitCommonControlsEx(core::ptr::addr_of_mut!(InitControls));

            hApplet = hinstDLL;
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
