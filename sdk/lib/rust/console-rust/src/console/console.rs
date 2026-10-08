// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_9896841D0A6A: [u16; 5] = [0x0022u16, 0x0025, 0x0073, 0x0022, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hApplet: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut ConInfo: PCONSOLE_STATE_INFO =
    ::core::ptr::null::<_CONSOLE_STATE_INFO>() as *mut _CONSOLE_STATE_INFO;

static mut SetConsoleInfo: BOOL = (FALSE) as i32;

static mut SaveConsoleInfo: BOOL = (FALSE) as i32;

// #define NDEBUG (valueless macro, no Rust constant emitted)

// Prototypes for OptionsProc, FontProc, LayoutProc, and ColorsProc are
// function declarations. They are provided by the function definitions in the crate.

unsafe extern "C" fn InitPropSheetPage(psp: *mut PROPSHEETPAGEW, idDlg: WORD, DlgProc: DLGPROC) {
    ZeroMemory!(psp as *mut core::ffi::c_void, core::mem::size_of::<PROPSHEETPAGEW>());
    (*psp).dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as DWORD;
    (*psp).dwFlags = PSP_DEFAULT;
    (*psp).hInstance = hApplet;
    (*psp).__bindgen_anon_1.pszTemplate = MAKEINTRESOURCEW!(idDlg);
    (*psp).pfnDlgProc = DlgProc;
    (*psp).lParam = 0;
}

unsafe extern "C" fn InitDefaultConsoleInfo(pConInfo: PCONSOLE_STATE_INFO) {
    ConCfgGetDefaultSettings(pConInfo);
}

unsafe extern "system" fn ApplyProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);

    match uMsg {
        WM_INITDIALOG => {
            CheckDlgButton(hwndDlg, (IDC_RADIO_APPLY_CURRENT as i32), BST_CHECKED);
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            if (LOWORD!(wParam)) as u32 == IDOK {
                if IsDlgButtonChecked(hwndDlg, (IDC_RADIO_APPLY_CURRENT as i32)) == BST_CHECKED {
                    EndDialog(hwndDlg, (IDC_RADIO_APPLY_CURRENT) as INT_PTR);
                } else {
                    EndDialog(hwndDlg, (IDC_RADIO_APPLY_ALL) as INT_PTR);
                }
            } else if (LOWORD!(wParam)) as u32 == IDCANCEL {
                EndDialog(hwndDlg, (IDCANCEL) as INT_PTR);
            }
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}

#[no_mangle]
pub unsafe extern "C" fn ApplyConsoleInfo(hwndDlg: HWND) {
    static mut ConsoleInfoAlreadySaved: BOOL = (FALSE) as i32;

    'done: {
        if ConsoleInfoAlreadySaved != 0 {
            break 'done;
        }

        if (*ConInfo).hWnd.is_null() {
            SetConsoleInfo = (FALSE) as i32;
            SaveConsoleInfo = (TRUE) as i32;
        } else {
            let res: INT_PTR = DialogBoxW!(
                hApplet,
                MAKEINTRESOURCEW!(IDD_APPLYOPTIONS),
                hwndDlg,
                Some(ApplyProc),
            );

            SetConsoleInfo = ((res) as u32 != (IDCANCEL) as _) as BOOL;
            SaveConsoleInfo = (res == (IDC_RADIO_APPLY_ALL) as INT_PTR) as BOOL;

            if SetConsoleInfo == 0 {
                SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, (PSNRET_INVALID_NOCHANGEPAGE) as LONG_PTR);
                return;
            }
        }

        ConsoleInfoAlreadySaved = (TRUE) as i32;
    }

    SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, (PSNRET_NOERROR) as LONG_PTR);
    return;
}

unsafe extern "system" fn PropSheetProc(hwndDlg: HWND, uMsg: UINT, lParam: LPARAM) -> i32 {
    let mut hIcon: HICON = core::ptr::null_mut();
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW!(IDC_CPLICON));
            SendMessageW(hwndDlg, WM_SETICON, (ICON_BIG) as WPARAM, hIcon as LPARAM);
        }
        _ => {}
    }
    return 0;
}

unsafe extern "system" fn InitApplet(hSectionOrWnd: HANDLE) -> LONG {
    let mut Result: INT_PTR = 0;
    let mut pSharedInfo: PCONSOLE_STATE_INFO = core::ptr::null_mut();
    let mut szTitle: [WCHAR; (MAX_PATH + 1) as _] = core::mem::zeroed();
    let mut psp: [PROPSHEETPAGEW; 4] = core::mem::zeroed();
    let mut psh: PROPSHEETHEADERW = core::mem::zeroed();
    let mut i: INT = 0 as INT;

    /*
     * Try to open the shared section via the handle parameter. If we succeed,
     * it means we were called by CONSRV for retrieving/setting parameters for
     * a given console. If we fail, it means we are retrieving/setting default
     * global parameters (and we were either called by CONSRV or directly by
     * the user via the Control Panel, etc...)
     */
    pSharedInfo = MapViewOfFile(
        hSectionOrWnd,
        FILE_MAP_READ,
        0,
        0,
        0,
    ) as PCONSOLE_STATE_INFO;
    if !pSharedInfo.is_null() {
        /*
         * We succeeded. We were called by CONSRV and are retrieving
         * parameters for a given console.
         */

        /* Copy the shared data into our allocated buffer */
        DPRINT1!(
            b"pSharedInfo->cbSize == %lu ; sizeof(core::mem::size_of::<CONSOLE_STATE_INFO>()) == %u\n\0" as *const u8
                as *const i8,
            (*pSharedInfo).cbSize,
            core::mem::size_of::<CONSOLE_STATE_INFO>() as u32,
        );

        /* Allocate a local buffer to hold console information */
        ConInfo = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (*pSharedInfo).cbSize as SIZE_T,
        ) as PCONSOLE_STATE_INFO;
        if !ConInfo.is_null() {
            RtlCopyMemory!(
                ConInfo as *mut core::ffi::c_void,
                pSharedInfo as *const core::ffi::c_void,
                (*pSharedInfo).cbSize as usize,
            );
        }

        /* Close the section */
        UnmapViewOfFile(pSharedInfo as *const core::ffi::c_void);
        CloseHandle(hSectionOrWnd);

        if ConInfo.is_null() {
            return 0 as LONG;
        }
    } else {
        /*
         * We failed. We are retrieving the default global parameters.
         */

        /* Allocate a local buffer to hold console information */
        ConInfo = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<CONSOLE_STATE_INFO>() as SIZE_T,
        ) as PCONSOLE_STATE_INFO;
        if ConInfo.is_null() {
            return 0 as LONG;
        }

        /*
         * Setting the console window handle to NULL indicates we are
         * retrieving/setting the default console parameters.
         */
        (*ConInfo).hWnd = core::ptr::null_mut();
        (*(*ConInfo).ConsoleTitle.as_mut_ptr().add(0 as usize)) = UNICODE_NULL;

        /* Use defaults */
        InitDefaultConsoleInfo(ConInfo);
    }

    /* Initialize the font support -- additional TrueType font cache and current preview font */
    InitTTFontCache();
    RefreshFontPreview(core::ptr::addr_of_mut!(FontPreview), ConInfo);

    /* Initialize the property sheet structure */
    ZeroMemory!(
        core::ptr::addr_of_mut!(psh) as *mut core::ffi::c_void,
        core::mem::size_of::<PROPSHEETHEADERW>(),
    );
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as DWORD;
    psh.dwFlags = PSH_PROPSHEETPAGE
        | PSH_PROPTITLE
        | /* PSH_USEHICON | */ PSH_USEICONID
        | PSH_NOAPPLYNOW
        | PSH_USECALLBACK;

    if (*(*ConInfo).ConsoleTitle.as_ptr().add(0 as usize)) != UNICODE_NULL {
        StringCchPrintfW!(
            core::ptr::addr_of_mut!(szTitle) as *mut WCHAR,
            ARRAYSIZE!(szTitle),
            RUSTCONV_WSTR_9896841D0A6A.as_ptr(),
            core::ptr::addr_of_mut!((*ConInfo).ConsoleTitle) as *mut WCHAR,
        );
        psh.pszCaption = core::ptr::addr_of_mut!(szTitle) as *mut WCHAR;
    } else {
        psh.pszCaption = MAKEINTRESOURCEW!(IDS_CPLNAME as WORD) as *mut WCHAR;
    }

    if !pSharedInfo.is_null() {
        /* We were started from a console window: this is our parent (or ConInfo->hWnd is NULL) */
        psh.hwndParent = (*ConInfo).hWnd;
    } else {
        /* We were started in another way (--> default parameters). Caller's window is our parent. */
        psh.hwndParent = hSectionOrWnd as HWND;
    }

    psh.hInstance = hApplet;
    // psh.hIcon = LoadIcon(hApplet, MAKEINTRESOURCEW(IDC_CPLICON));
    psh.__bindgen_anon_1.pszIcon = MAKEINTRESOURCEW!(IDC_CPLICON as WORD) as *mut WCHAR;
    psh.nPages = ARRAYSIZE!(psp) as UINT;
    psh.__bindgen_anon_2.nStartPage = 0 as UINT;
    psh.__bindgen_anon_3.ppsp = core::ptr::addr_of_mut!(psp) as *mut PROPSHEETPAGEW;
    psh.pfnCallback = Some(PropSheetProc);

    let fresh_a = i;
    i = i + 1;
    InitPropSheetPage(
        (core::ptr::addr_of_mut!(psp) as *mut PROPSHEETPAGEW).offset(fresh_a as isize),
        IDD_PROPPAGEOPTIONS as WORD,
        Some(OptionsProc),
    );
    let fresh_b = i;
    i = i + 1;
    InitPropSheetPage(
        (core::ptr::addr_of_mut!(psp) as *mut PROPSHEETPAGEW).offset(fresh_b as isize),
        IDD_PROPPAGEFONT as WORD,
        Some(FontProc),
    );
    let fresh_c = i;
    i = i + 1;
    InitPropSheetPage(
        (core::ptr::addr_of_mut!(psp) as *mut PROPSHEETPAGEW).offset(fresh_c as isize),
        IDD_PROPPAGELAYOUT as WORD,
        Some(LayoutProc),
    );
    let fresh_d = i;
    i = i + 1;
    InitPropSheetPage(
        (core::ptr::addr_of_mut!(psp) as *mut PROPSHEETPAGEW).offset(fresh_d as isize),
        IDD_PROPPAGECOLORS as WORD,
        Some(ColorsProc),
    );

    /* Display the property sheet */
    RegisterWinPrevClass(hApplet);
    Result = PropertySheetW(core::ptr::addr_of_mut!(psh));
    UnRegisterWinPrevClass(hApplet);

    /* Clear the font support */
    ResetFontPreview!(core::ptr::addr_of_mut!(FontPreview));
    ClearTTFontCache();

    'Quit: {
        /* Apply the console settings if necessary */
        if SetConsoleInfo != 0 {
            let mut hSection: HANDLE = core::ptr::null_mut();

            /*
             * Create a memory section to share with CONSRV, and map it.
             */
            hSection = CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                core::ptr::null_mut(),
                PAGE_READWRITE,
                0,
                (*ConInfo).cbSize as DWORD,
                core::ptr::null(),
            );
            if hSection.is_null() {
                DPRINT1!(
                    b"Error when creating file mapping, error = %d\n\0" as *const u8 as *const i8,
                    GetLastError(),
                );
                break 'Quit;
            }

            pSharedInfo = MapViewOfFile(
                hSection,
                FILE_MAP_ALL_ACCESS,
                0,
                0,
                0,
            ) as PCONSOLE_STATE_INFO;
            if pSharedInfo.is_null() {
                DPRINT1!(
                    b"Error when mapping view of file, error = %d\n\0" as *const u8 as *const i8,
                    GetLastError(),
                );
                CloseHandle(hSection);
                break 'Quit;
            }

            /* Copy the console information into the section and unmap it */
            RtlCopyMemory!(
                pSharedInfo as *mut core::ffi::c_void,
                ConInfo as *const core::ffi::c_void,
                (*ConInfo).cbSize as usize,
            );
            UnmapViewOfFile(pSharedInfo as *const core::ffi::c_void);

            /*
             * Signal to CONSRV that it can apply the new settings.
             *
             * NOTE: SetConsoleInfo set to TRUE by ApplyConsoleInfo()
             * *only* when ConInfo->hWnd != NULL and the user did not
             * press IDCANCEL in the confirmation dialog.
             */
            SendMessageW(
                (*ConInfo).hWnd,
                (WM_SETCONSOLEINFO) as UINT,
                hSection as WPARAM,
                0,
            );

            /* Close the section and return */
            CloseHandle(hSection);
        }

        /* Save the console settings */
        if SaveConsoleInfo != 0 {
            /* Default settings saved when ConInfo->hWnd == NULL */
            ConCfgWriteUserSettings(ConInfo, ((*ConInfo).hWnd == core::ptr::null_mut()) as BOOLEAN);
        }
    }

    /* Cleanup */
    HeapFree(
        GetProcessHeap(),
        0,
        ConInfo as PVOID,
    );
    ConInfo = core::ptr::null_mut();

    return (Result != -(1 as INT_PTR)) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn CPlApplet(
    hwndCPl: HWND,
    uMsg: UINT,
    lParam1: LPARAM,
    lParam2: LPARAM,
) -> LONG {
    if uMsg == CPL_INIT {
        return (TRUE) as LONG;
    } else if uMsg == CPL_EXIT {
        // TODO: Free allocated memory
    } else if uMsg == CPL_GETCOUNT {
        return 1;
    } else if uMsg == CPL_INQUIRE {
        let CPlInfo: *mut CPLINFO = lParam2 as *mut CPLINFO;
        (*CPlInfo).idIcon = (IDC_CPLICON as i32);
        (*CPlInfo).idName = (IDS_CPLNAME as i32);
        (*CPlInfo).idInfo = (IDS_CPLDESCRIPTION as i32);
    } else if uMsg == CPL_DBLCLK {
        InitApplet(hwndCPl as HANDLE);
    }

    return (FALSE) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: DWORD,
    lpvReserved: LPVOID,
) -> INT {
    UNREFERENCED_PARAMETER!(lpvReserved);

    match dwReason {
        DLL_PROCESS_ATTACH => {
            hApplet = hinstDLL;
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
        }
        _ => {}
    }

    return (TRUE) as INT;
}
