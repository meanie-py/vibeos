use crate::umandlg::process::CloseProcess;
use crate::umandlg::process::IsProcessRunning;
use crate::umandlg::process::LaunchProcess;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_5601D8A07BC8: [u16; 8] = [0x0055u16, 0x0074, 0x0069, 0x006C, 0x006D, 0x0061, 0x006E, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_8FBA7B702ED0: [u16; 8] = [0x006Fu16, 0x0073, 0x006B, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_A5A979EF2F26: [u16; 12] = [
    0x006Du16, 0x0061, 0x0067, 0x006E, 0x0069, 0x0066, 0x0079, 0x002E, 0x0065, 0x0078,
    0x0065, 0x0000,
];

#[no_mangle]
pub static mut Globals: UTILMAN_GLOBALS = UTILMAN_GLOBALS {
    hInstance: ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__,
    hIcon: ::core::ptr::null::<HICON__>() as *mut HICON__,
    iTimer: 0,
    iSelectedIndex: 0,
    hDlgCtlStart: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hDlgCtlStop: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hListDlg: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hMainDlg: ::core::ptr::null::<HWND__>() as *mut HWND__,
    szRunning: [0; 256],
    szNotRunning: [0; 256],
    szGrpBoxTitle: [0; 256],
};

#[no_mangle]
pub static mut EntriesList: [UTILMAN_STATE; 2] = unsafe {
    [
        _UTILMAN_STATE {
            lpszProgram: RUSTCONV_C2R_WSTR_A5A979EF2F26.as_ptr() as LPCWSTR,
            uNameId: IDS_MAGNIFIER as UINT,
            szResource: ::core::mem::transmute::<
                [u8; 512],
                [WCHAR; 256],
            >(
                *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            bState: (FALSE as i32),
        },
        _UTILMAN_STATE {
            lpszProgram: RUSTCONV_C2R_WSTR_8FBA7B702ED0.as_ptr() as LPCWSTR,
            uNameId: IDS_OSK as UINT,
            szResource: ::core::mem::transmute::<
                [u8; 512],
                [WCHAR; 256],
            >(
                *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            bState: (FALSE as i32),
        },
    ]
};

// Field names below are assumed; they must match the UTILMAN_STATE definition.
#[no_mangle]
pub unsafe extern "C" fn InitUtilsList(bInitGui: BOOL) {
    let mut i: UINT = 0;
    if bInitGui == 0 {
        i = 0 as UINT;
        while (i as usize) < _countof!(EntriesList) as usize {
            LoadStringW(
                Globals.hInstance,
                EntriesList[i as usize].uNameId,
                core::ptr::addr_of_mut!(EntriesList[i as usize].szResource) as LPWSTR,
                _countof!(EntriesList[i as usize].szResource) as ::core::ffi::c_int,
            );
            EntriesList[i as usize].bState = FALSE as BOOL;
            i = i.wrapping_add(1);
        }
    } else {
        let mut iItem: INT = 0;
        let mut bIsRunning: BOOL = 0;
        let mut szFormat: [WCHAR; (MAX_BUFFER) as _] = [0; (MAX_BUFFER) as _];
        SendMessageW(
            Globals.hListDlg,
            LB_RESETCONTENT as UINT,
            0 as WPARAM,
            0 as LPARAM,
        );
        i = 0 as UINT;
        while (i as usize) < _countof!(EntriesList) as usize {
            bIsRunning = IsProcessRunning(EntriesList[i as usize].lpszProgram);
            EntriesList[i as usize].bState = bIsRunning;
            StringCchPrintfW!(
                core::ptr::addr_of_mut!(szFormat) as *mut WCHAR,
                _countof!(szFormat) as usize,
                (if bIsRunning != 0 {
                    core::ptr::addr_of_mut!(Globals.szRunning) as *mut WCHAR
                } else {
                    core::ptr::addr_of_mut!(Globals.szNotRunning) as *mut WCHAR
                }) as *const WCHAR,
                core::ptr::addr_of_mut!(EntriesList[i as usize].szResource) as *mut WCHAR,
            );
            iItem = SendMessageW(
                Globals.hListDlg,
                LB_ADDSTRING as UINT,
                0 as WPARAM,
                core::ptr::addr_of_mut!(szFormat) as *mut WCHAR as LPARAM,
            ) as INT;
            if iItem != LB_ERR as INT {
                SendMessageW(
                    Globals.hListDlg,
                    LB_SETITEMDATA as UINT,
                    iItem as WPARAM,
                    core::ptr::addr_of_mut!(EntriesList[i as usize]) as LPARAM,
                );
            }
            i = i.wrapping_add(1);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn DlgInitHandler(hDlg: HWND) -> BOOL {
    let mut PosX: INT = 0;
    let mut PosY: INT = 0;
    let mut rc: RECT = core::mem::zeroed();
    let mut szAboutDlg: [WCHAR; (MAX_BUFFER) as _] = [0; (MAX_BUFFER) as _];
    let mut szAppPath: [WCHAR; (MAX_BUFFER) as _] = [0; (MAX_BUFFER) as _];
    let mut hSysMenu: HMENU;

    /* Save the dialog handle */
    Globals.hMainDlg = hDlg;

    /* Center the dialog on the screen */
    GetWindowRect(hDlg, core::ptr::addr_of_mut!(rc));
    PosX = (GetSystemMetrics((SM_CXSCREEN as i32)) - rc.right) / 2;
    PosY = (GetSystemMetrics((SM_CYSCREEN as i32)) - rc.bottom) / 2;
    SetWindowPos(hDlg, core::ptr::null_mut(), PosX, PosY, 0, 0, SWP_NOZORDER | SWP_NOSIZE);

    /* Extract the icon resource from the executable process */
    GetModuleFileNameW(core::ptr::null_mut(), szAppPath.as_mut_ptr(), _countof!(szAppPath));
    Globals.hIcon = ExtractIconW(Globals.hInstance, szAppPath.as_mut_ptr(), 0);

    /* Set the icon within the dialog's title bar */
    if !Globals.hIcon.is_null() {
        SendMessageW(hDlg, WM_SETICON, (ICON_SMALL) as WPARAM, Globals.hIcon as LPARAM);
    }

    /* Retrieve the system menu and append the "About" menu item onto it */
    hSysMenu = GetSystemMenu(hDlg, (FALSE) as BOOL);
    if !hSysMenu.is_null() {
        if LoadStringW(Globals.hInstance, IDM_ABOUT, szAboutDlg.as_mut_ptr(), (_countof!(szAboutDlg)) as i32) != 0 {
            AppendMenuW(hSysMenu, MF_SEPARATOR, 0, core::ptr::null());
            AppendMenuW(hSysMenu, MF_STRING, (IDM_ABOUT) as UINT_PTR, szAboutDlg.as_mut_ptr());
        }
    }

    /* Get the dialog items, specifically the dialog list box, the Start and Stop buttons */
    Globals.hListDlg = GetDlgItem(hDlg, (IDC_LISTBOX as i32));
    Globals.hDlgCtlStart = GetDlgItem(hDlg, (IDC_START as i32));
    Globals.hDlgCtlStop = GetDlgItem(hDlg, (IDC_STOP as i32));

    /* Initialize the GUI listbox */
    InitUtilsList((TRUE) as BOOL);

    /* Set the selection to the first item */
    Globals.iSelectedIndex = 0;

    /* Refresh the list */
    ListBoxRefreshContents();

    /* Create a timer, we'll use it to control the state of our items in the listbox */
    Globals.iTimer = SetTimer(hDlg, 0, 400, None);

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ShowAboutDlg(hDlgParent: HWND) {
    let mut szApp: [WCHAR; (MAX_BUFFER) as _] = [0; (MAX_BUFFER) as _];
    let mut szAuthors: [WCHAR; (MAX_BUFFER) as _] = [0; (MAX_BUFFER) as _];

    LoadStringW(
        Globals.hInstance,
        IDS_APP_NAME as UINT,
        &raw mut szApp as LPWSTR,
        (_countof!(szApp)) as i32,
    );
    LoadStringW(
        Globals.hInstance,
        IDS_AUTHORS as UINT,
        &raw mut szAuthors as LPWSTR,
        (_countof!(szAuthors)) as i32,
    );

    ShellAboutW(
        hDlgParent,
        &raw mut szApp as *mut WCHAR as LPCWSTR,
        &raw mut szAuthors as *mut WCHAR as LPCWSTR,
        Globals.hIcon,
    );
}
#[no_mangle]
pub unsafe extern "C" fn GroupBoxUpdateTitle() {
    let mut szFormat: [WCHAR; (MAX_BUFFER) as _] = [0; (MAX_BUFFER) as _];

    /* Format the string with the utility's name and set it to the listbox's title */
    StringCchPrintfW!(
        core::ptr::addr_of_mut!(szFormat) as STRSAFE_LPWSTR,
        _countof!(szFormat),
        core::ptr::addr_of_mut!(Globals.szGrpBoxTitle) as *mut WCHAR as STRSAFE_LPCWSTR,
        core::ptr::addr_of_mut!(
            (*(core::ptr::addr_of_mut!(EntriesList) as *mut UTILMAN_STATE)
                .offset(Globals.iSelectedIndex as isize))
            .szResource
        ) as *mut WCHAR,
    );
    SetWindowTextW(
        GetDlgItem(Globals.hMainDlg, (IDC_GROUPBOX as i32)),
        core::ptr::addr_of_mut!(szFormat) as *mut WCHAR as LPCWSTR,
    );
}
#[no_mangle]
pub unsafe extern "C" fn UpdateUtilityState(bUtilState: BOOL) {
    Button_Enable!(Globals.hDlgCtlStart, (bUtilState == 0) as BOOL);
    Button_Enable!(Globals.hDlgCtlStop, bUtilState);

    /* Update the groupbox's title based on the selected utility item */
    GroupBoxUpdateTitle();
}
#[no_mangle]
pub unsafe extern "C" fn ListBoxRefreshContents() -> INT {
    let mut i: UINT = 0;
    let mut iItem: INT = 0;
    let mut bIsRunning: BOOL = 0;
    let mut szFormat: [WCHAR; (MAX_BUFFER) as _] = [0; (MAX_BUFFER) as _];

    /* Disable listbox redraw */
    SendMessageW(
        Globals.hListDlg,
        WM_SETREDRAW as UINT,
        FALSE as WPARAM,
        0 as LPARAM,
    );

    i = 0 as UINT;
    while (i as usize) < (_countof!(EntriesList) as usize) {
        /* Check the utility's state */
        bIsRunning = IsProcessRunning(EntriesList[i as usize].lpszProgram);
        if bIsRunning != EntriesList[i as usize].bState {
            /* The utility's state has changed, save it */
            EntriesList[i as usize].bState = bIsRunning;

            /* Update the corresponding item in the listbox */
            StringCchPrintfW!(
                szFormat.as_mut_ptr(),
                _countof!(szFormat) as size_t,
                (if bIsRunning != 0 {
                    core::ptr::addr_of_mut!(Globals.szRunning) as *mut WCHAR
                } else {
                    core::ptr::addr_of_mut!(Globals.szNotRunning) as *mut WCHAR
                }) as STRSAFE_LPCWSTR,
                core::ptr::addr_of_mut!(EntriesList[i as usize].szResource) as *mut WCHAR,
            );

            SendMessageW(
                Globals.hListDlg,
                LB_DELETESTRING as UINT,
                (i as LPARAM) as WPARAM,
                0 as LPARAM,
            );
            iItem = SendMessageW(
                Globals.hListDlg,
                LB_INSERTSTRING as UINT,
                (i as LPARAM) as WPARAM,
                szFormat.as_mut_ptr() as LPARAM,
            ) as INT;
            if iItem != LB_ERR as INT {
                SendMessageW(
                    Globals.hListDlg,
                    LB_SETITEMDATA as UINT,
                    iItem as WPARAM,
                    core::ptr::addr_of_mut!(EntriesList[i as usize]) as LPARAM,
                );
            }
        }
        i = i.wrapping_add(1);
    }

    /* Re-enable listbox redraw */
    SendMessageW(
        Globals.hListDlg,
        WM_SETREDRAW as UINT,
        TRUE as WPARAM,
        0 as LPARAM,
    );

    /*
     * Check the previously selected item. This will help us determine what
     * item has been selected and set its focus selection back. Furthermore, check
     * the state of each accessibility tool and enable/disable the buttons.
     */
    SendMessageW(
        Globals.hListDlg,
        LB_SETCURSEL as UINT,
        Globals.iSelectedIndex as WPARAM,
        0 as LPARAM,
    );
    UpdateUtilityState(EntriesList[Globals.iSelectedIndex as usize].bState);

    return 0 as INT;
}
#[no_mangle]
pub unsafe extern "system" fn DlgProc(
    hDlg: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match Msg {
        WM_INITDIALOG => {
            DlgInitHandler(hDlg);
            return (TRUE) as INT_PTR;
        }
        WM_CLOSE => {
            KillTimer(hDlg, Globals.iTimer);
            DestroyIcon(Globals.hIcon);
            EndDialog(hDlg, (FALSE) as INT_PTR);
        }
        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDC_OK | IDC_CANCEL => {
                    EndDialog(hDlg, (FALSE) as INT_PTR);
                }
                IDC_LISTBOX => {
                    match (HIWORD!(wParam) as ::core::ffi::c_int) as u32 {
                        LBN_SELCHANGE => {
                            /* Retrieve the index of the current selected item */
                            let mut iIndex: INT = (SendMessageW(
                                Globals.hListDlg,
                                LB_GETCURSEL,
                                0,
                                0,
                            )) as i32;
                            if !((iIndex == LB_ERR)
                                || ((iIndex as usize) >= _countof!(EntriesList) as usize))
                            {
                                /* Assign the selected index and check the utility's state */
                                Globals.iSelectedIndex = iIndex;
                                UpdateUtilityState(
                                    EntriesList[Globals.iSelectedIndex as usize].bState,
                                );
                            }
                        }
                        _ => {}
                    }
                }
                IDC_START => {
                    LaunchProcess(
                        EntriesList[Globals.iSelectedIndex as usize].lpszProgram,
                    );
                }
                IDC_STOP => {
                    CloseProcess(
                        EntriesList[Globals.iSelectedIndex as usize].lpszProgram,
                    );
                }
                _ => {}
            }
        }
        WM_TIMER => {
            ListBoxRefreshContents();
            return 0;
        }
        WM_SYSCOMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDM_ABOUT => {
                    ShowAboutDlg(hDlg);
                }
                _ => {}
            }
        }
        _ => {}
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn UManStartDlg() -> BOOL {
    let mut hMutex: HANDLE = core::ptr::null_mut();
    let mut dwError: DWORD = 0;
    let mut iccex: INITCOMMONCONTROLSEX = core::mem::zeroed();

    /* Create a mutant object for the program. */
    hMutex = CreateMutexW(core::ptr::null_mut(), (FALSE) as BOOL, RUSTCONV_WSTR_5601D8A07BC8.as_ptr());
    if !hMutex.is_null() {
        /* Check if there's already a mutex for the program */
        dwError = GetLastError();
        if dwError == ERROR_ALREADY_EXISTS {
            /*
                The program's instance is already here. That means
                the program is running and we should not set a new instance
                and mutex object.
            */
            CloseHandle(hMutex);
            return (FALSE) as BOOL;
        }
    }

    /* Load the common controls for the program */
    iccex.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
    iccex.dwICC = (ICC_STANDARD_CLASSES | ICC_WIN95_CLASSES) as DWORD;
    InitCommonControlsEx(core::ptr::addr_of_mut!(iccex));

    LoadStringW(Globals.hInstance, IDS_RUNNING,
                core::ptr::addr_of_mut!(Globals.szRunning) as LPWSTR, _countof!(Globals.szRunning) as core::ffi::c_int);
    LoadStringW(Globals.hInstance, IDS_NOTRUNNING,
                core::ptr::addr_of_mut!(Globals.szNotRunning) as LPWSTR, _countof!(Globals.szNotRunning) as core::ffi::c_int);
    LoadStringW(Globals.hInstance, IDS_GROUPBOX_OPTIONS_TITLE,
                core::ptr::addr_of_mut!(Globals.szGrpBoxTitle) as LPWSTR, _countof!(Globals.szGrpBoxTitle) as core::ffi::c_int);

    /* Initialize the list of accessibility utilities */
    InitUtilsList((FALSE) as BOOL);

    /* Create the dialog box of the program */
    DialogBoxW!(Globals.hInstance,
               MAKEINTRESOURCEW!(IDD_MAIN_DIALOG),
               GetDesktopWindow(),
               Some(DlgProc));

    /* Delete the mutex */
    if !hMutex.is_null() {
        CloseHandle(hMutex);
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hDllInstance: HINSTANCE,
    fdwReason: DWORD,
    lpvReserved: LPVOID,
) -> BOOL {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            /* We don't care for DLL_THREAD_ATTACH and DLL_THREAD_DETACH notifications */
            DisableThreadLibraryCalls(hDllInstance);

            /* Initialize the globals */
            ZeroMemory!(core::ptr::addr_of_mut!(Globals) as *mut core::ffi::c_void, core::mem::size_of::<UTILMAN_GLOBALS>());
            Globals.hInstance = hDllInstance;
        }

        DLL_PROCESS_DETACH => {}

        _ => {}
    }

    return (TRUE) as BOOL;
}
