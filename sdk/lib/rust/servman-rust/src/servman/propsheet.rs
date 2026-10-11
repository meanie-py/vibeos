use crate::servman::propsheet_depends::DependenciesPageProc;
use crate::servman::propsheet_general::GeneralPageProc;
use crate::servman::propsheet_logon::LogonPageProc;
use crate::servman::propsheet_recovery::RecoveryPageProc;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// `PropSheetThread` is only forward-declared here. Its definition is emitted with the function bodies, so no declaration is needed in the preamble.

unsafe extern "C" fn InitPropSheetPage(
    psp: *mut PROPSHEETPAGE,
    dlgInfo: PSERVICEPROPSHEET,
    idDlg: WORD,
    DlgProc: DLGPROC,
) {
    ZeroMemory!(
        psp as *mut core::ffi::c_void,
        core::mem::size_of::<PROPSHEETPAGE>() as SIZE_T,
    );
    (*psp).dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    (*psp).dwFlags = PSP_DEFAULT as DWORD;
    (*psp).hInstance = hInstance;
    (*psp).__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(idDlg) as LPCWSTR;
    (*psp).pfnDlgProc = DlgProc;
    (*psp).lParam = dlgInfo as LPARAM;
}
#[no_mangle]
pub unsafe extern "C" fn OpenPropSheet(mut Info: PMAIN_WND_INFO) {
    let mut pServicePropSheet: PSERVICEPROPSHEET = core::ptr::null_mut::<_SERVICEPROPSHEET>();
    let mut hThread: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    pServicePropSheet = HeapAlloc(
        ProcessHeap,
        0,
        core::mem::size_of::<_SERVICEPROPSHEET>() as SIZE_T,
    ) as PSERVICEPROPSHEET;
    if pServicePropSheet.is_null() {
        return;
    }
    (*pServicePropSheet).pService = (*Info).pCurrentService;
    (*pServicePropSheet).Info = Info;
    hThread = _beginthreadex(
        core::ptr::null_mut(),
        0,
        Some(
            PropSheetThread,
        ),
        pServicePropSheet as *mut core::ffi::c_void,
        0,
        core::ptr::null_mut::<core::ffi::c_uint>(),
    ) as HANDLE;
    if !hThread.is_null() {
        CloseHandle(hThread);
    }
}
#[no_mangle]
pub unsafe extern "system" fn PropSheetThread(Param: *mut ::core::ffi::c_void) -> ::core::ffi::c_uint {
    let mut pServicePropSheet: PSERVICEPROPSHEET = core::ptr::null_mut();
    let mut psh: PROPSHEETHEADER = core::mem::zeroed();
    let mut psp: [PROPSHEETPAGE; 4] = core::mem::zeroed();
    let mut hDlg: HWND = core::ptr::null_mut();
    let mut Msg: MSG = core::mem::zeroed();

    pServicePropSheet = Param as PSERVICEPROPSHEET;

    ZeroMemory!(
        core::ptr::addr_of_mut!(psh) as *mut ::core::ffi::c_void,
        core::mem::size_of::<PROPSHEETHEADER>(),
    );
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADER>() as DWORD;
    psh.dwFlags = (PSH_PROPSHEETPAGE | PSH_PROPTITLE | PSH_MODELESS) as DWORD;
    psh.hwndParent = (*(*pServicePropSheet).Info).hMainWnd;
    psh.hInstance = hInstance;
    psh.__bindgen_anon_1.hIcon = LoadIcon(hInstance, MAKEINTRESOURCE!(IDI_SM_ICON));
    psh.pszCaption = (*(*(*pServicePropSheet).Info).pCurrentService).lpDisplayName;
    psh.nPages = (core::mem::size_of::<[PROPSHEETPAGE; 4]>()
        / core::mem::size_of::<PROPSHEETPAGE>()) as UINT;
    psh.__bindgen_anon_2.nStartPage = 0 as UINT;
    psh.__bindgen_anon_3.ppsp = core::ptr::addr_of_mut!(psp) as *mut PROPSHEETPAGE;

    /* Initialize the tabs */
    InitPropSheetPage(
        core::ptr::addr_of_mut!(psp[0]),
        pServicePropSheet,
        (IDD_DLG_GENERAL) as WORD,
        Some(GeneralPageProc),
    );
    InitPropSheetPage(
        core::ptr::addr_of_mut!(psp[1]),
        pServicePropSheet,
        (IDD_LOGON) as WORD,
        Some(LogonPageProc),
    );
    InitPropSheetPage(
        core::ptr::addr_of_mut!(psp[2]),
        pServicePropSheet,
        (IDD_RECOVERY) as WORD,
        Some(RecoveryPageProc),
    );
    InitPropSheetPage(
        core::ptr::addr_of_mut!(psp[3]),
        pServicePropSheet,
        (IDD_DLG_DEPEND) as WORD,
        Some(DependenciesPageProc),
    );

    hDlg = PropertySheetW(core::ptr::addr_of_mut!(psh)) as HWND;
    if !hDlg.is_null() {
        /* Pump the message queue */
        while GetMessageW(
            core::ptr::addr_of_mut!(Msg),
            core::ptr::null_mut(),
            0,
            0,
        ) != 0
        {
            if PropSheet_GetCurrentPageHwnd!(hDlg).is_null() {
                /* The user hit the ok / cancel button, pull it down */
                EnableWindow((*(*pServicePropSheet).Info).hMainWnd, (TRUE) as BOOL);
                DestroyWindow(hDlg);
            }

            if PropSheet_IsDialogMessage!(hDlg, core::ptr::addr_of_mut!(Msg)) == 0 {
                TranslateMessage(core::ptr::addr_of_mut!(Msg));
                DispatchMessageW(core::ptr::addr_of_mut!(Msg));
            }
        }
    }

    HeapFree(
        GetProcessHeap(),
        0,
        pServicePropSheet as PVOID,
    );

    return (!hDlg.is_null()) as ::core::ffi::c_uint;
}
