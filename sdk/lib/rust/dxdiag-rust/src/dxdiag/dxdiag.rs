use crate::dxdiag::help::HelpPageWndProc;
use crate::dxdiag::sound::InitializeDirectSoundPage;
use crate::dxdiag::display::InitializeDisplayAdapters;
use crate::dxdiag::input::InputPageWndProc;
use crate::dxdiag::music::MusicPageWndProc;
use crate::dxdiag::network::NetworkPageWndProc;
use crate::dxdiag::system::SystemPageWndProc;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hInst: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut hTabCtrlWnd: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

pub const ETDT_DISABLE: u32 = 0x00000001;
pub const ETDT_ENABLE: u32 = 0x00000002;
pub const ETDT_USETABTEXTURE: u32 = 0x00000004;
pub const ETDT_ENABLETAB: u32 = ETDT_ENABLE | ETDT_USETABTEXTURE;
pub type ETDTProc = unsafe extern "system" fn(HWND, u32) -> i32;
#[no_mangle]
pub unsafe extern "C" fn EnableDialogTheme(hwnd: HWND) -> BOOL {
    let mut hUXTheme: HMODULE;
    let mut fnEnableThemeDialogTexture: Option<ETDTProc>;

    hUXTheme = LoadLibraryA(b"uxtheme.dll\0".as_ptr() as *const i8) as HMODULE;

    if !hUXTheme.is_null() {
        fnEnableThemeDialogTexture = core::mem::transmute(GetProcAddress(
            hUXTheme as HINSTANCE,
            b"EnableThemeDialogTexture\0".as_ptr() as *const i8,
        ));

        if fnEnableThemeDialogTexture.is_some() {
            fnEnableThemeDialogTexture.unwrap_unchecked()(hwnd, ETDT_ENABLETAB as DWORD);

            FreeLibrary(hUXTheme);
            return (TRUE) as BOOL;
        } else {
            // Failed to locate API!
            FreeLibrary(hUXTheme);
            return (FALSE) as BOOL;
        }
    } else {
        // Not running under XP? Just fail gracefully
        return (FALSE) as BOOL;
    }
}
#[no_mangle]
pub unsafe extern "C" fn DestroyTabCtrlDialogs(pContext: PDXDIAG_CONTEXT) {
    let mut Index: UINT = 0;

    /* destroy default dialogs */
    while (Index as usize)
        < (core::mem::size_of::<[HWND; 5]>() as usize)
            / (core::mem::size_of::<HWND>() as usize)
    {
        if !(*pContext).hDialogs[Index as usize].is_null() {
            DestroyWindow((*pContext).hDialogs[Index as usize]);
        }
        Index = Index.wrapping_add(1);
    }

    /* destroy display dialogs */
    Index = 0;
    while (Index as ULONG) < (*pContext).NumDisplayAdapter {
        if !(**(*pContext).DisplayAdapters.offset(Index as isize))
            .hDisplayWnd
            .is_null()
        {
            DestroyWindow((**(*pContext).DisplayAdapters.offset(Index as isize)).hDisplayWnd);
        }
        Index = Index.wrapping_add(1);
    }

    /* destroy audio dialogs */
    Index = 0;
    while (Index as ULONG) < (*pContext).NumSoundAdapter {
        if !(*(*pContext).hSoundWnd.offset(Index as isize)).is_null() {
            DestroyWindow(*(*pContext).hSoundWnd.offset(Index as isize));
        }
        Index = Index.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn InsertTabCtrlItem(hDlgCtrl: HWND, Position: INT, uId: LPWSTR) {
    let mut szName: [WCHAR; 100] = [0; 100];
    let mut item: TCITEMW = core::mem::zeroed();

    /* setup item info */
    memset(
        core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<TCITEMW>(),
    );
    item.mask = TCIF_TEXT as UINT;

    /* load item name */
    if HIWORD!(uId as usize) == 0 {
        szName[0 as usize] = 0x0000u16;
        if LoadStringW(
            hInst,
            LOWORD!(uId as usize) as UINT,
            szName.as_mut_ptr(),
            100 as INT,
        ) == 0
        {
            return;
        }
        szName[99 as usize] = 0x0000u16;
        item.pszText = szName.as_mut_ptr();
    } else {
        item.pszText = uId;
    }

    SendMessageW(
        hDlgCtrl,
        TCM_INSERTITEM as UINT,
        Position as WPARAM,
        core::ptr::addr_of_mut!(item) as LPARAM,
    );
}
#[no_mangle]
pub unsafe extern "C" fn TabCtrl_OnSelChange(mut pContext: PDXDIAG_CONTEXT) {
    let mut Index: INT = 0;
    let mut CurSel: INT = 0;
    CurSel = TabCtrl_GetCurSel!(hTabCtrlWnd);
    if CurSel < 0
        || CurSel as ULONG
            > (*pContext)
                .NumDisplayAdapter
                .wrapping_add((*pContext).NumSoundAdapter)
                .wrapping_add(5 as ULONG)
    {
        return;
    }
    Index = 0;
    while Index < 5 {
        ShowWindow((*pContext).hDialogs[Index as usize], (SW_HIDE) as i32);
        Index += 1;
    }
    Index = 0;
    while (Index as ULONG) < (*pContext).NumDisplayAdapter {
        ShowWindow(
            (**(*pContext).DisplayAdapters.offset(Index as isize)).hDisplayWnd,
            (SW_HIDE) as i32,
        );
        Index += 1;
    }
    Index = 0;
    while (Index as ULONG) < (*pContext).NumSoundAdapter {
        ShowWindow(*(*pContext).hSoundWnd.offset(Index as isize), (SW_HIDE) as i32);
        Index += 1;
    }
    if CurSel == 0
        || CurSel as ULONG
            > (*pContext)
                .NumDisplayAdapter
                .wrapping_add((*pContext).NumSoundAdapter)
    {
        if CurSel != 0 {
            CurSel = (CurSel as ULONG).wrapping_sub(
                (*pContext)
                    .NumDisplayAdapter
                    .wrapping_add((*pContext).NumSoundAdapter),
            ) as INT;
        }
        // The C bound is `>` where `>=` was meant: CurSel == adapters + 5 indexes
        // hDialogs[5], one past the array. Show nothing for an index that is not a page.
        if let Some(&hPage) = (*pContext).hDialogs.get(CurSel as usize) {
            ShowWindow(hPage, (SW_SHOW) as i32);
        }
        return;
    }
    if ((CurSel - 1) as ULONG) < (*pContext).NumDisplayAdapter {
        ShowWindow(
            (**(*pContext)
                .DisplayAdapters
                .offset((CurSel - 1) as isize))
            .hDisplayWnd,
            (SW_SHOW) as i32,
        );
        return;
    }
    CurSel = (CurSel as ULONG).wrapping_sub(
        (*pContext)
            .NumDisplayAdapter
            .wrapping_add(1 as ULONG),
    ) as INT;
    ShowWindow(*(*pContext).hSoundWnd.offset(CurSel as isize), (SW_SHOW) as i32);
}
#[no_mangle]
pub unsafe extern "C" fn InitializeTabCtrl(hwndDlg: HWND, pContext: PDXDIAG_CONTEXT) {
    /* get tabctrl */
    hTabCtrlWnd = GetDlgItem(hwndDlg, (IDC_TAB_CONTROL as i32));
    (*pContext).hTabCtrl = hTabCtrlWnd;

    /* create the dialogs */
    (*pContext).hDialogs[0 as usize] = CreateDialogParamW(
        hInst,
        MAKEINTRESOURCEW!(IDD_SYSTEM_DIALOG),
        (*pContext).hTabCtrl,
        Some(SystemPageWndProc),
        pContext as LPARAM,
    );
    EnableDialogTheme((*pContext).hDialogs[0 as usize]);

    (*pContext).hDialogs[1 as usize] = CreateDialogParamW(
        hInst,
        MAKEINTRESOURCEW!(IDD_MUSIC_DIALOG),
        (*pContext).hTabCtrl,
        Some(MusicPageWndProc),
        pContext as LPARAM,
    );
    EnableDialogTheme((*pContext).hDialogs[1 as usize]);

    (*pContext).hDialogs[2 as usize] = CreateDialogParamW(
        hInst,
        MAKEINTRESOURCEW!(IDD_INPUT_DIALOG),
        (*pContext).hTabCtrl,
        Some(InputPageWndProc),
        pContext as LPARAM,
    );
    EnableDialogTheme((*pContext).hDialogs[2 as usize]);

    (*pContext).hDialogs[3 as usize] = CreateDialogParamW(
        hInst,
        MAKEINTRESOURCEW!(IDD_NETWORK_DIALOG),
        (*pContext).hTabCtrl,
        Some(NetworkPageWndProc),
        pContext as LPARAM,
    );
    EnableDialogTheme((*pContext).hDialogs[3 as usize]);

    (*pContext).hDialogs[4 as usize] = CreateDialogParamW(
        hInst,
        MAKEINTRESOURCEW!(IDD_HELP_DIALOG),
        (*pContext).hTabCtrl,
        Some(HelpPageWndProc),
        pContext as LPARAM,
    );
    EnableDialogTheme((*pContext).hDialogs[4 as usize]);

    /* insert tab ctrl items */
    InsertTabCtrlItem(
        hTabCtrlWnd,
        0 as INT,
        MAKEINTRESOURCEW!(IDS_SYSTEM_DIALOG),
    );
    InitializeDisplayAdapters(pContext);
    InitializeDirectSoundPage(pContext);
    InsertTabCtrlItem(
        hTabCtrlWnd,
        (*pContext)
            .NumDisplayAdapter
            .wrapping_add((*pContext).NumSoundAdapter)
            .wrapping_add(1 as ULONG) as INT,
        MAKEINTRESOURCEW!(IDS_MUSIC_DIALOG),
    );
    InsertTabCtrlItem(
        hTabCtrlWnd,
        (*pContext)
            .NumDisplayAdapter
            .wrapping_add((*pContext).NumSoundAdapter)
            .wrapping_add(2 as ULONG) as INT,
        MAKEINTRESOURCEW!(IDS_INPUT_DIALOG),
    );
    InsertTabCtrlItem(
        hTabCtrlWnd,
        (*pContext)
            .NumDisplayAdapter
            .wrapping_add((*pContext).NumSoundAdapter)
            .wrapping_add(3 as ULONG) as INT,
        MAKEINTRESOURCEW!(IDS_NETWORK_DIALOG),
    );
    InsertTabCtrlItem(
        hTabCtrlWnd,
        (*pContext)
            .NumDisplayAdapter
            .wrapping_add((*pContext).NumSoundAdapter)
            .wrapping_add(4 as ULONG) as INT,
        MAKEINTRESOURCEW!(IDS_HELP_DIALOG),
    );
    TabCtrl_OnSelChange(pContext);
}
#[no_mangle]
pub unsafe extern "C" fn InitializeDxDiagDialog(hwndDlg: HWND) {
    let mut pContext: PDXDIAG_CONTEXT;
    let mut hIcon: HICON;

    pContext = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<DXDIAG_CONTEXT>() as SIZE_T,
    ) as PDXDIAG_CONTEXT;
    if pContext.is_null() {
        return;
    }

    /* store window handle */
    (*pContext).hMainDialog = hwndDlg;

    /* store the context */
    SetWindowLongPtr(
        hwndDlg,
        DWLP_USER as ::core::ffi::c_int,
        pContext as LONG_PTR,
    );

    /* initialize the tab ctrl */
    InitializeTabCtrl(hwndDlg, pContext);

    /* load application icon */
    hIcon = LoadImageW(
        hInst,
        MAKEINTRESOURCEW!(IDI_APPICON) as LPCWSTR,
        IMAGE_ICON as UINT,
        16 as ::core::ffi::c_int,
        16 as ::core::ffi::c_int,
        0 as UINT,
    ) as HICON;
    if hIcon.is_null() {
        return;
    }

    /* display icon */
    SendMessage(
        hwndDlg,
        WM_SETICON as UINT,
        ICON_SMALL as WPARAM,
        hIcon as LPARAM,
    );
}

#[no_mangle]
pub unsafe extern "system" fn DxDiagWndProc(hwndDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pnmh: LPNMHDR;
    let mut pContext: PDXDIAG_CONTEXT;

    pContext = GetWindowLongPtr(hwndDlg, DWLP_USER) as PDXDIAG_CONTEXT;

    match message {
        WM_INITDIALOG => {
            InitializeDxDiagDialog(hwndDlg);
            return (TRUE) as INT_PTR;
        }
        WM_COMMAND => {
            if LOWORD!(wParam) == (IDC_BUTTON_SAVE_INFO as u16) {
                //TODO
                /* handle save information */
                return (TRUE) as INT_PTR;
            }

            if LOWORD!(wParam) == (IDC_BUTTON_NEXT as u16) {
                let mut CurSel: INT;

                /* retrieve current page */
                CurSel = TabCtrl_GetCurSel!(hTabCtrlWnd);
                CurSel += 1;

                /* enable/disable next button */
                EnableWindow(
                    GetDlgItem(hwndDlg, (IDC_BUTTON_NEXT as i32)),
                    (CurSel != TabCtrl_GetItemCount!(hTabCtrlWnd) - 1) as BOOL,
                );

                /* switch to next tab */
                SendMessageW(hTabCtrlWnd, TCM_SETCURSEL, CurSel as WPARAM, 0);

                /* show next page */
                TabCtrl_OnSelChange(pContext);
                return (TRUE) as INT_PTR;
            }

            if LOWORD!(wParam) == (IDC_BUTTON_HELP as u16) {
                //TODO
                /* handle help button */
                return (TRUE) as INT_PTR;
            }

            if (LOWORD!(wParam)) as u32 == IDCANCEL || LOWORD!(wParam) == (IDC_BUTTON_EXIT as u16) {
                EndDialog(hwndDlg, (LOWORD!(wParam)) as INT_PTR);
                return (TRUE) as INT_PTR;
            }
        }
        WM_NOTIFY => {
            pnmh = lParam as LPNMHDR;
            if ((*pnmh).hwndFrom == hTabCtrlWnd)
                && ((*pnmh).idFrom == (IDC_TAB_CONTROL as _))
                && ((*pnmh).code == (TCN_SELCHANGE) as u32)
            {
                let mut CurSel: INT = TabCtrl_GetCurSel!(hTabCtrlWnd);

                /* enable/disable next button */
                EnableWindow(
                    GetDlgItem(hwndDlg, (IDC_BUTTON_NEXT as i32)),
                    (CurSel != TabCtrl_GetItemCount!(hTabCtrlWnd) - 1) as BOOL,
                );

                TabCtrl_OnSelChange(pContext);
            }
        }
        WM_DESTROY => {
            DestroyTabCtrlDialogs(pContext);
            return DefWindowProc(hwndDlg, message, wParam, lParam);
        }
        _ => {}
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpCmdLine: LPWSTR,
    nCmdShow: i32,
) -> i32 {
    let mut InitControls: INITCOMMONCONTROLSEX = core::mem::zeroed();

    UNREFERENCED_PARAMETER!(hPrevInstance);
    UNREFERENCED_PARAMETER!(lpCmdLine);
    UNREFERENCED_PARAMETER!(nCmdShow);

    InitControls.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
    InitControls.dwICC = (ICC_TAB_CLASSES | ICC_LISTVIEW_CLASSES | ICC_STANDARD_CLASSES | ICC_TREEVIEW_CLASSES) as DWORD;
    InitCommonControlsEx(core::ptr::addr_of_mut!(InitControls));

    hInst = hInstance;

    DialogBox!(
        hInst,
        MAKEINTRESOURCE!(IDD_MAIN_DIALOG),
        core::ptr::null_mut(),
        Some(DxDiagWndProc),
    );

    return 0;
}
