use crate::servman::misc::AllocAndLoadString;
use crate::servman::stop_dependencies::CreateStopDependsDialog;
use crate::servman::control::DoControlService;
use crate::servman::start::DoStartService;
use crate::servman::stop::DoStopService;
use crate::servman::stop_dependencies::GetListOfServicesToStop;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct PROGRESS_DATA {
    pub hDlg: HWND,
    pub hProgress: HWND,
    pub ServiceName: LPWSTR,
    pub Action: ULONG,
    pub StopDepends: BOOL,
    pub ServiceList: LPWSTR,
    pub Param: PVOID,
}

pub type PPROGRESS_DATA = *mut PROGRESS_DATA;

pub const PROGRESS_RANGE: i32 = 20;

pub const PROGRESS_STEP_MAX: i32 = 15;
#[no_mangle]
pub unsafe extern "C" fn ShowError(dwLastError: DWORD) {
    let mut lpMsg: LPWSTR = core::ptr::null_mut();

    if FormatMessageW(
        (FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS) as DWORD,
        core::ptr::null(),
        dwLastError,
        LANG_USER_DEFAULT as DWORD,
        core::ptr::addr_of_mut!(lpMsg) as LPWSTR,
        0 as DWORD,
        core::ptr::null_mut(),
    ) == 0
    {
        return;
    }

    MessageBoxW(
        core::ptr::null_mut(),
        lpMsg as LPCWSTR,
        core::ptr::null(),
        (MB_OK | MB_ICONERROR) as UINT,
    );
    LocalFree(lpMsg as *mut core::ffi::c_void);
}

unsafe extern "C" fn ResetProgressDialog(
    mut hDlg: HWND,
    mut ServiceName: LPWSTR,
    mut LabelId: ULONG,
) {
    let mut lpProgStr: LPWSTR = core::ptr::null_mut();
    /* Load the label Id */
    if AllocAndLoadString(core::ptr::addr_of_mut!(lpProgStr), hInstance, LabelId as UINT) != 0 {
        /* Write it to the dialog */
        SendDlgItemMessageW(
            hDlg,
            (IDC_SERVCON_INFO as i32),
            WM_SETTEXT as UINT,
            0 as WPARAM,
            lpProgStr as LPARAM,
        );

        LocalFree(lpProgStr as HLOCAL);
    }

    /* Write the service name to the dialog */
    SendDlgItemMessageW(
        hDlg,
        (IDC_SERVCON_NAME as i32),
        WM_SETTEXT as UINT,
        0 as WPARAM,
        ServiceName as LPARAM,
    );

    /* Set the progress bar to the start */
    SendDlgItemMessageW(
        hDlg,
        (IDC_SERVCON_PROGRESS as i32),
        PBM_SETPOS as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ActionThread(Param: *mut ::core::ffi::c_void) -> ::core::ffi::c_uint {
    let ProgressData: PPROGRESS_DATA = Param as PPROGRESS_DATA;
    let mut dwResult: DWORD;

    if (*ProgressData).Action == ACTION_START {
        /* Setup the progress dialog for this action */
        ResetProgressDialog(
            (*ProgressData).hDlg,
            (*ProgressData).ServiceName,
            IDS_PROGRESS_INFO_START,
        );

        /* Start the service */
        dwResult = DoStartService(
            (*ProgressData).ServiceName,
            (*ProgressData).hProgress as HANDLE,
            (*ProgressData).Param as LPWSTR,
        );
        if dwResult == ERROR_SUCCESS {
            /* We're done, slide the progress bar up to the top */
            CompleteProgressBar(((*ProgressData).hProgress).cast());
        } else {
            ShowError(dwResult);
        }
    } else if (*ProgressData).Action == ACTION_STOP
        || (*ProgressData).Action == ACTION_RESTART
    {
        /* Check if there are and dependants to stop */
        if (*ProgressData).StopDepends != 0 && !(*ProgressData).ServiceList.is_null() {
            let mut lpStr: LPWSTR = (*ProgressData).ServiceList;

            /* Loop through all the services in the list */
            loop {
                /* Break when we hit the double null */
                if *lpStr == 0x0000u16
                    && *lpStr.offset(1 as isize) == 0x0000u16
                {
                    break;
                }

                /* If this isn't our first time in the loop we'll
                have been left on a null char */
                if *lpStr == 0x0000u16 {
                    lpStr = lpStr.offset(1);
                }

                ResetProgressDialog(
                    (*ProgressData).hDlg,
                    lpStr,
                    IDS_PROGRESS_INFO_STOP,
                );

                /* Stop the requested service */
                dwResult = DoStopService(
                    (*ProgressData).ServiceName,
                    ((*ProgressData).hProgress).cast(),
                );
                if dwResult == ERROR_SUCCESS {
                    CompleteProgressBar(((*ProgressData).hProgress).cast());
                } else {
                    ShowError(dwResult);
                }

                /* Move onto the next string */
                while *lpStr != 0x0000u16 {
                    lpStr = lpStr.offset(1);
                }
            }
        }

        ResetProgressDialog(
            (*ProgressData).hDlg,
            (*ProgressData).ServiceName,
            IDS_PROGRESS_INFO_STOP,
        );

        dwResult = DoStopService(
            (*ProgressData).ServiceName,
            ((*ProgressData).hProgress).cast(),
        );
        if dwResult == ERROR_SUCCESS {
            CompleteProgressBar(((*ProgressData).hProgress).cast());
        } else {
            ShowError(dwResult);
        }

        /* If this was a restart, we'll need to start the service back up */
        if (*ProgressData).Action == ACTION_RESTART {
            /* Setup the progress dialog for this action */
            ResetProgressDialog(
                (*ProgressData).hDlg,
                (*ProgressData).ServiceName,
                IDS_PROGRESS_INFO_START,
            );

            /* Start the service */
            dwResult = DoStartService(
                (*ProgressData).ServiceName,
                ((*ProgressData).hProgress).cast(),
                ::core::ptr::null_mut(),
            );
            if dwResult == ERROR_SUCCESS {
                /* We're done, slide the progress bar up to the top */
                CompleteProgressBar(((*ProgressData).hProgress).cast());
            } else {
                ShowError(dwResult);
            }
        }
    } else if (*ProgressData).Action == ACTION_PAUSE {
        /* Setup the progress dialog for this action */
        ResetProgressDialog(
            (*ProgressData).hDlg,
            (*ProgressData).ServiceName,
            IDS_PROGRESS_INFO_PAUSE,
        );

        /* Pause the service */
        dwResult = DoControlService(
            (*ProgressData).ServiceName,
            (*ProgressData).hProgress,
            SERVICE_CONTROL_PAUSE,
        );
        if dwResult == ERROR_SUCCESS {
            /* We're done, slide the progress bar up to the top */
            CompleteProgressBar(((*ProgressData).hProgress).cast());
        } else {
            ShowError(dwResult);
        }
    } else if (*ProgressData).Action == ACTION_RESUME {
        /* Setup the progress dialog for this action */
        ResetProgressDialog(
            (*ProgressData).hDlg,
            (*ProgressData).ServiceName,
            IDS_PROGRESS_INFO_RESUME,
        );

        /* resume the service */
        dwResult = DoControlService(
            (*ProgressData).ServiceName,
            (*ProgressData).hProgress,
            SERVICE_CONTROL_CONTINUE,
        );
        if dwResult == ERROR_SUCCESS {
            /* We're done, slide the progress bar up to the top */
            CompleteProgressBar(((*ProgressData).hProgress).cast());
        } else {
            ShowError(dwResult);
        }
    }

    EndDialog((*ProgressData).hDlg, (IDOK) as INT_PTR);

    _endthreadex(0);
    return 0;
}

unsafe extern "C" fn InitProgressDialog(
    mut hDlg: HWND,
    mut Message: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> BOOL {
    let mut ProgressData: PPROGRESS_DATA = lParam as PPROGRESS_DATA;
    let mut hThread: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();

    (*ProgressData).hDlg = hDlg;

    /* Get a handle to the progress bar */
    (*ProgressData).hProgress = GetDlgItem(hDlg, (IDC_SERVCON_PROGRESS as i32));
    if (*ProgressData).hProgress.is_null() {
        return (FALSE) as BOOL;
    }

    /* Set the progress bar range */
    SendMessageW(
        (*ProgressData).hProgress,
        PBM_SETRANGE as UINT,
        0 as WPARAM,
        MAKELPARAM!(0 as WORD, PROGRESS_RANGE as WORD),
    );

    /* Set the progress bar step */
    SendMessageW(
        (*ProgressData).hProgress,
        PBM_SETSTEP as UINT,
        1 as WPARAM,
        0 as LPARAM,
    );

    /* Create a thread to handle the service control */
    hThread = _beginthreadex(
        core::ptr::null_mut(),
        0 as ::core::ffi::c_uint,
        Some(ActionThread),
        ProgressData as *mut core::ffi::c_void,
        0 as ::core::ffi::c_uint,
        core::ptr::null_mut::<core::ffi::c_uint>(),
    ) as HANDLE;
    if hThread.is_null() {
        return (FALSE) as BOOL;
    }

    CloseHandle(hThread);

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn ProgressDialogProc(
    hDlg: HWND,
    Message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match Message {
        WM_INITDIALOG => {
            return (InitProgressDialog(hDlg, Message, wParam, lParam)) as INT_PTR;
        }
        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDOK => {
                    EndDialog(hDlg, (wParam) as INT_PTR);
                }
                _ => {}
            }
        }
        _ => {
            return (FALSE) as INT_PTR;
        }
    }

    return (TRUE) as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn CompleteProgressBar(hProgress: HANDLE) {
    let hProgBar: HWND = hProgress as HWND;
    let mut Pos_local: UINT = 0 as UINT;

    /* Get the current position */
    Pos_local = SendMessageW(hProgBar,
                       PBM_GETPOS as UINT,
                       0 as WPARAM,
                       0 as LPARAM) as UINT;

    /* Loop until we hit the max */
    while Pos_local <= PROGRESS_RANGE as UINT
    {
        /* Increment the progress bar */
        SendMessageW(hProgBar,
                     PBM_DELTAPOS as UINT,
                     Pos_local as WPARAM,
                     0 as LPARAM);

        /* Wait for 15ms to give it a smooth feel */
        Sleep(15 as DWORD);
        Pos_local = Pos_local.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn IncrementProgressBar(hProgress: HANDLE, Step: UINT) {
    let hProgBar: HWND = hProgress as HWND;
    let mut Position: UINT = 0;

    /* Don't allow the progress to reach to complete*/
    Position = SendMessageW(hProgBar, PBM_GETPOS as UINT, 0 as WPARAM, 0 as LPARAM) as UINT;
    if Position < PROGRESS_STEP_MAX as UINT {
        /* Do we want to increment the default amount? */
        if Step == DEFAULT_STEP as UINT {
            /* Use the step value we set on create */
            SendMessageW(hProgBar, PBM_STEPIT as UINT, 0 as WPARAM, 0 as LPARAM);
        } else {
            /* Use the value passed */
            SendMessageW(hProgBar, PBM_SETPOS as UINT, Step as WPARAM, 0 as LPARAM);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn RunActionWithProgress(mut hParent: HWND, mut ServiceName: LPWSTR, mut DisplayName: LPWSTR, mut Action: UINT, mut Param: PVOID) -> BOOL {
    let mut ProgressData: PROGRESS_DATA = core::mem::zeroed();
    let mut ServiceList: LPWSTR = core::ptr::null_mut();
    let mut StopDepends: BOOL = 0;
    let mut Result: INT_PTR = 0;

    StopDepends = FALSE as BOOL;
    ServiceList = core::ptr::null_mut();

    /* Check if we'll be stopping the service */
    if Action == ACTION_STOP as UINT || Action == ACTION_RESTART as UINT
    {
        /* Does the service have any dependent services which need stopping first */
        ServiceList = GetListOfServicesToStop(ServiceName);
        if !ServiceList.is_null()
        {
            /* Ask the user if they want to stop the dependants */
            StopDepends = CreateStopDependsDialog(hParent, ServiceName, DisplayName, ServiceList);

            /* Exit early if the user decided not to stop the dependants */
            if StopDepends == FALSE as BOOL
            {
                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    ServiceList as PVOID,
                );
                return FALSE as BOOL;
            }
        }
    }

    ProgressData.hDlg = core::ptr::null_mut();
    ProgressData.ServiceName = ServiceName;
    ProgressData.Action = Action as ULONG;
    ProgressData.StopDepends = StopDepends;
    ProgressData.ServiceList = ServiceList;
    ProgressData.Param = Param;

    Result = DialogBoxParamW(
        hInstance,
        MAKEINTRESOURCEW!(IDD_DLG_PROGRESS) as LPCWSTR,
        hParent,
        Some(ProgressDialogProc),
        core::ptr::addr_of_mut!(ProgressData) as LPARAM,
    );

    if !ServiceList.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            ServiceList as PVOID,
        );
    }

    return (Result == IDOK as INT_PTR) as BOOL;
}
