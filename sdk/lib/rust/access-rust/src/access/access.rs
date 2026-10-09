use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hApplet: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut Applets: [APPLET; 1] = unsafe {
    [_APPLET {
        idIcon: (IDI_CPLACCESS as i32),
        idName: (IDS_CPLSYSTEMNAME as i32),
        idDescription: (IDS_CPLSYSTEMDESCRIPTION as i32),
        AppletProc: Some(SystemApplet),
    }]
};

pub const NUM_APPLETS: usize = 1;

// Resource IDs IDI_CPLACCESS, IDS_CPLSYSTEMNAME and IDS_CPLSYSTEMDESCRIPTION are
// from the access.cpl resource header and must be defined by the caller.
// Field names follow the C APPLET initializer order: icon, name, description, proc.

unsafe extern "C" fn ReadSettings(pGlobalData: PGLOBAL_DATA) -> BOOL {
    let mut dwDisposition: DWORD = 0;
    let mut dwLength: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lError: LONG = 0;

    /* Get sticky keys information */
    (*pGlobalData).stickyKeys.cbSize = core::mem::size_of::<STICKYKEYS>() as DWORD;
    if SystemParametersInfo(
        SPI_GETSTICKYKEYS,
        core::mem::size_of::<STICKYKEYS>() as UINT,
        core::ptr::addr_of_mut!((*pGlobalData).stickyKeys) as PVOID,
        0,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    /* Get filter keys information */
    (*pGlobalData).filterKeys.cbSize = core::mem::size_of::<FILTERKEYS>() as UINT;
    if SystemParametersInfo(
        SPI_GETFILTERKEYS,
        core::mem::size_of::<FILTERKEYS>() as UINT,
        core::ptr::addr_of_mut!((*pGlobalData).filterKeys) as PVOID,
        0,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    /* Get toggle keys information */
    (*pGlobalData).toggleKeys.cbSize = core::mem::size_of::<TOGGLEKEYS>() as DWORD;
    if SystemParametersInfo(
        SPI_GETTOGGLEKEYS,
        core::mem::size_of::<TOGGLEKEYS>() as UINT,
        core::ptr::addr_of_mut!((*pGlobalData).toggleKeys) as PVOID,
        0,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    /* Get keyboard preference information */
    if SystemParametersInfo(
        SPI_GETKEYBOARDPREF,
        0,
        core::ptr::addr_of_mut!((*pGlobalData).bKeyboardPref) as PVOID,
        0,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    /* Get high contrast information */
    (*pGlobalData).highContrast.cbSize = core::mem::size_of::<HIGHCONTRAST>() as UINT;
    SystemParametersInfo(
        SPI_GETHIGHCONTRAST,
        core::mem::size_of::<HIGHCONTRAST>() as UINT,
        core::ptr::addr_of_mut!((*pGlobalData).highContrast) as PVOID,
        0,
    );

    SystemParametersInfo(
        SPI_GETCARETWIDTH,
        0,
        core::ptr::addr_of_mut!((*pGlobalData).uCaretWidth) as PVOID,
        0,
    );

    (*pGlobalData).uCaretBlinkTime = GetCaretBlinkTime();

    /* Get sound settings */
    (*pGlobalData).ssSoundSentry.cbSize = core::mem::size_of::<SOUNDSENTRY>() as UINT;
    SystemParametersInfo(
        SPI_GETSOUNDSENTRY,
        core::mem::size_of::<SOUNDSENTRY>() as UINT,
        core::ptr::addr_of_mut!((*pGlobalData).ssSoundSentry) as PVOID,
        0,
    );

    SystemParametersInfo(
        SPI_GETSHOWSOUNDS,
        0,
        core::ptr::addr_of_mut!((*pGlobalData).bShowSounds) as PVOID,
        0,
    );

    /* Get mouse keys information */
    (*pGlobalData).mouseKeys.cbSize = core::mem::size_of::<MOUSEKEYS>() as UINT;
    SystemParametersInfo(
        SPI_GETMOUSEKEYS,
        core::mem::size_of::<MOUSEKEYS>() as UINT,
        core::ptr::addr_of_mut!((*pGlobalData).mouseKeys) as PVOID,
        0,
    );

    /* Get access timeout information */
    (*pGlobalData).accessTimeout.cbSize = core::mem::size_of::<ACCESSTIMEOUT>() as UINT;
    SystemParametersInfo(
        SPI_GETACCESSTIMEOUT,
        core::mem::size_of::<ACCESSTIMEOUT>() as UINT,
        core::ptr::addr_of_mut!((*pGlobalData).accessTimeout) as PVOID,
        0,
    );

    /* Get serial keys information */
    (*pGlobalData).serialKeys.cbSize = core::mem::size_of::<SERIALKEYS>() as UINT;
    (*pGlobalData).serialKeys.lpszActivePort =
        (core::ptr::addr_of_mut!((*pGlobalData).szActivePort) as PWSTR).cast();
    (*pGlobalData).serialKeys.lpszPort = (core::ptr::addr_of_mut!((*pGlobalData).szPort) as PWSTR).cast();
    SystemParametersInfo(
        SPI_GETSERIALKEYS,
        core::mem::size_of::<SERIALKEYS>() as UINT,
        core::ptr::addr_of_mut!((*pGlobalData).serialKeys) as PVOID,
        0,
    );

    (*pGlobalData).bWarningSounds = (TRUE) as i32;
    (*pGlobalData).bSoundOnActivation = (TRUE) as i32;

    let ctrlPanelPath: [u16; 28] = [
        'C' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16,
        ' ' as u16, 'P' as u16, 'a' as u16, 'n' as u16, 'e' as u16, 'l' as u16, '\\' as u16,
        'A' as u16, 'c' as u16, 'c' as u16, 'e' as u16, 's' as u16, 's' as u16, 'i' as u16,
        'b' as u16, 'i' as u16, 'l' as u16, 'i' as u16, 't' as u16, 'y' as u16, 0,
    ];

    lError = RegCreateKeyEx(
        HKEY_CURRENT_USER,
        ctrlPanelPath.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        (KEY_EXECUTE | KEY_QUERY_VALUE),
        core::ptr::null_mut(),
        &mut hKey,
        &mut dwDisposition,
    );
    if (lError) as u32 != ERROR_SUCCESS {
        return (TRUE) as BOOL;
    }

    dwLength = core::mem::size_of::<BOOL>() as DWORD;
    let warningSoundsName: [u16; 15] = [
        'W' as u16, 'a' as u16, 'r' as u16, 'n' as u16, 'i' as u16, 'n' as u16, 'g' as u16,
        ' ' as u16, 'S' as u16, 'o' as u16, 'u' as u16, 'n' as u16, 'd' as u16, 's' as u16, 0,
    ];
    lError = RegQueryValueEx(
        hKey,
        warningSoundsName.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*pGlobalData).bWarningSounds) as LPBYTE,
        &mut dwLength,
    );
    if (lError) as u32 != ERROR_SUCCESS {
        (*pGlobalData).bWarningSounds = (TRUE) as i32;
    }

    dwLength = core::mem::size_of::<BOOL>() as DWORD;
    let soundOnActivationName: [u16; 20] = [
        'S' as u16, 'o' as u16, 'u' as u16, 'n' as u16, 'd' as u16, ' ' as u16, 'O' as u16,
        'n' as u16, ' ' as u16, 'A' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'v' as u16,
        'a' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 0,
    ];
    lError = RegQueryValueEx(
        hKey,
        soundOnActivationName.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*pGlobalData).bSoundOnActivation) as LPBYTE,
        &mut dwLength,
    );
    if (lError) as u32 != ERROR_SUCCESS {
        (*pGlobalData).bSoundOnActivation = (TRUE) as i32;
    }

    RegCloseKey(hKey);

    return (TRUE) as BOOL;
}

unsafe extern "C" fn InitPropSheetPage(psp: *mut PROPSHEETPAGE, idDlg: WORD, DlgProc: DLGPROC, pGlobalData: PGLOBAL_DATA) {
    ZeroMemory!(psp as *mut core::ffi::c_void, core::mem::size_of::<PROPSHEETPAGE>());
    (*psp).dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    (*psp).dwFlags = PSP_DEFAULT;
    (*psp).hInstance = hApplet;
    (*psp).__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(idDlg);
    (*psp).pfnDlgProc = DlgProc;
    (*psp).lParam = pGlobalData as LPARAM;
}

unsafe extern "system" fn PropSheetProc(hwndDlg: HWND, uMsg: UINT, lParam: LPARAM) -> i32 {
    let mut hIcon: HICON = core::ptr::null_mut();
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW!(IDI_CPLACCESS));
            SendMessageW(hwndDlg, WM_SETICON, (ICON_BIG) as WPARAM, hIcon as LPARAM);
        }
        _ => {}
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn SystemApplet(hwnd: HWND, uMsg: UINT, wParam: LPARAM, lParam: LPARAM) -> LONG {
    let mut pGlobalData: PGLOBAL_DATA;
    let mut psp: [PROPSHEETPAGE; 5] = core::mem::zeroed();
    let mut psh: PROPSHEETHEADER = core::mem::zeroed();
    let mut nPage: UINT = 0 as UINT; // This is unsigned so we don't have to deal with negative numbers
    let mut ret: INT;

    if uMsg == CPL_STARTWPARMSW as UINT && lParam != 0 as LPARAM {
        nPage = ((*(lParam as PWSTR)) as i32 - '1' as i32) as UINT; // Convert from 1-based to 0-based.
    }

    pGlobalData = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<GLOBAL_DATA>() as SIZE_T,
    ) as PGLOBAL_DATA;
    if pGlobalData.is_null() {
        return 0 as LONG;
    }

    if ReadSettings(pGlobalData) == 0 {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            pGlobalData as PVOID,
        );
        return 0 as LONG;
    }

    ZeroMemory!(
        &raw mut psh as PVOID,
        core::mem::size_of::<PROPSHEETHEADER>(),
    );
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADER>() as DWORD;
    psh.dwFlags = (PSH_PROPSHEETPAGE | PSH_USEICONID | PSH_USECALLBACK) as DWORD;
    psh.hwndParent = hwnd;
    psh.hInstance = hApplet;
    psh.__bindgen_anon_1.pszIcon = MAKEINTRESOURCEW!(IDI_CPLACCESS as WORD);
    psh.pszCaption = MAKEINTRESOURCEW!(IDS_CPLSYSTEMNAME as WORD);
    psh.nPages = (core::mem::size_of::<[PROPSHEETPAGE; 5]>()
        / core::mem::size_of::<PROPSHEETPAGE>()) as UINT;
    psh.__bindgen_anon_2.nStartPage = 0 as UINT;
    psh.__bindgen_anon_3.ppsp = &raw mut psp as *mut PROPSHEETPAGE as LPCPROPSHEETPAGE;
    psh.pfnCallback = Some(PropSheetProc);

    InitPropSheetPage(
        &raw mut psp[0 as usize],
        IDD_PROPPAGEKEYBOARD as WORD,
        Some(KeyboardPageProc),
        pGlobalData,
    );
    InitPropSheetPage(
        &raw mut psp[1 as usize],
        IDD_PROPPAGESOUND as WORD,
        Some(SoundPageProc),
        pGlobalData,
    );
    InitPropSheetPage(
        &raw mut psp[2 as usize],
        IDD_PROPPAGEDISPLAY as WORD,
        Some(DisplayPageProc),
        pGlobalData,
    );
    InitPropSheetPage(
        &raw mut psp[3 as usize],
        IDD_PROPPAGEMOUSE as WORD,
        Some(MousePageProc),
        pGlobalData,
    );
    InitPropSheetPage(
        &raw mut psp[4 as usize],
        IDD_PROPPAGEGENERAL as WORD,
        Some(GeneralPageProc),
        pGlobalData,
    );

    if nPage != 0 as UINT && nPage <= psh.nPages {
        psh.__bindgen_anon_2.nStartPage = nPage;
    }

    ret = PropertySheet(&raw mut psh) as INT;

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        pGlobalData as PVOID,
    );

    return (ret != -(1 as INT)) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn CPlApplet(
    hwndCPl: HWND,
    uMsg: UINT,
    lParam1: LPARAM,
    lParam2: LPARAM,
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
                let CPlInfo: *mut CPLINFO = lParam2 as *mut CPLINFO;
                (*CPlInfo).lData = 0 as LONG_PTR;
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
                    hwndCPl, uMsg, lParam1, lParam2
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
                    hwndCPl, uMsg, lParam1, lParam2
                );
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
        _ if dwReason == DLL_PROCESS_ATTACH || dwReason == DLL_THREAD_ATTACH => {
            hApplet = hinstDLL;
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
