use crate::deskmon::shxiface::IDeskMonitor_InitIface;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

static mut bFoundAdapter: BOOL = 0;

static mut diAdapter: DEVINST = 0;

pub const FUNC_DEVICEPROPERTIES: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"DevicePropertiesW\0")
};

// #define NDEBUG (empty object-like macro, no value to emit)

// UNICODE branch selected
pub type PDEVICEPROPERTIES = unsafe extern "system" fn(HWND, LPCWSTR, LPCWSTR, BOOL) -> INT_PTR;

unsafe extern "C" fn GetMonitorDevInstID(lpDeviceID: LPCTSTR) -> LPTSTR {
    let mut cr: CONFIGRET = 0;
    let mut diChild: DEVINST = 0;
    let mut szProperty: [TCHAR; 256] = [0; 256];
    let mut dwSize: DWORD = 0;
    let mut lpDevInstId: LPTSTR = core::ptr::null_mut();

    if ((bFoundAdapter) == 0) {
        return core::ptr::null_mut();
    }

    /* Look for child monitor devices of selected video adapter */
    cr = CM_Get_Child(core::ptr::addr_of_mut!(diChild), diAdapter, 0);
    if cr != CR_SUCCESS {
        DPRINT1!("CM_Get_Child failed: %d\n", cr);
        return core::ptr::null_mut();
    }

    'dowhile: loop {
        'body: {
            /* Get Hardware ID for each of them */
            dwSize = (core::mem::size_of_val(&szProperty)
                - core::mem::size_of::<TCHAR>()) as DWORD;

            cr = CM_Get_DevNode_Registry_Property(
                diChild,
                CM_DRP_HARDWAREID,
                core::ptr::null_mut(),
                (szProperty.as_mut_ptr()).cast(),
                core::ptr::addr_of_mut!(dwSize),
                0,
            );
            if cr != CR_SUCCESS {
                DPRINT1!("CM_Get_DevNode_Registry_Property failed: %d\n", cr);
                break 'body;
            }

            /* Concatenate with driver key */
            _tcscat(szProperty.as_mut_ptr(), (['\\' as u16, 0u16].as_ptr()));
            dwSize = (core::mem::size_of_val(&szProperty)
                - core::mem::size_of::<TCHAR>()) as DWORD;
            dwSize -= (_tcslen(szProperty.as_mut_ptr()) as DWORD)
                * (core::mem::size_of::<TCHAR>() as DWORD);

            cr = CM_Get_DevNode_Registry_Property(
                diChild,
                CM_DRP_DRIVER,
                core::ptr::null_mut(),
                (szProperty
                    .as_mut_ptr()
                    .add(_tcslen(szProperty.as_mut_ptr()) as usize)).cast(),
                core::ptr::addr_of_mut!(dwSize),
                0,
            );
            if cr != CR_SUCCESS {
                DPRINT1!("CM_Get_DevNode_Registry_Property failed: %d\n", cr);
                break 'body;
            }

            /* If the strings match, this is our monitor device node */
            if _tcscmp(szProperty.as_mut_ptr(), lpDeviceID) == 0 {
                cr = CM_Get_Device_ID_Size(
                    core::ptr::addr_of_mut!(dwSize),
                    diChild,
                    0,
                );
                if cr != CR_SUCCESS {
                    DPRINT1!("CM_Get_Device_ID_Size failed: %d\n", cr);
                    break 'dowhile;
                }

                lpDevInstId = LocalAlloc(
                    LMEM_FIXED,
                    ((dwSize + 1) as SIZE_T)
                        * (core::mem::size_of::<TCHAR>() as SIZE_T),
                ) as LPTSTR;
                if lpDevInstId.is_null() {
                    DPRINT1!("LocalAlloc failed\n");
                    break 'dowhile;
                }

                cr = CM_Get_Device_ID(
                    diChild,
                    lpDevInstId,
                    dwSize + 1,
                    0,
                );
                if cr != CR_SUCCESS {
                    DPRINT1!("CM_Get_Device_ID failed: %d\n", cr);
                    LocalFree(lpDevInstId as HLOCAL);
                    lpDevInstId = core::ptr::null_mut();
                }

                break 'dowhile;
            }
        }
        if !(CM_Get_Sibling(
            core::ptr::addr_of_mut!(diChild),
            diChild,
            0,
        ) == CR_SUCCESS)
        {
            break 'dowhile;
        }
    }

    return lpDevInstId;
}

unsafe extern "C" fn ShowMonitorProperties(This: PDESKMONITOR) {
    let mut hDevMgr: HMODULE;
    let mut pDeviceProperties: Option<PDEVICEPROPERTIES>;
    let mut lpDevInstID: LPTSTR;
    if !(*This).SelMonitor.is_null() {
        lpDevInstID = GetMonitorDevInstID(
            &raw mut (*(*This).SelMonitor).dd.DeviceID as *mut WCHAR as LPCTSTR,
        );
        if !lpDevInstID.is_null() {
            let devmgr: [u16; 11] = [
                'd' as u16, 'e' as u16, 'v' as u16, 'm' as u16, 'g' as u16, 'r' as u16,
                '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0,
            ];
            hDevMgr = LoadLibrary(devmgr.as_ptr() as LPCWSTR) as HMODULE;
            if !hDevMgr.is_null() {
                pDeviceProperties =
                    ::core::mem::transmute::<FARPROC, Option<PDEVICEPROPERTIES>>(
                        GetProcAddress(
                            hDevMgr as HINSTANCE,
                            (FUNC_DEVICEPROPERTIES).as_ptr(),
                        ),
                    );
                if pDeviceProperties.is_some() {
                    let __devprop_fn = pDeviceProperties
                        .unwrap_unchecked();
                    __devprop_fn(
                        (*This).hwndDlg,
                        ::core::ptr::null::<WCHAR>(),
                        lpDevInstID as LPCWSTR,
                        FALSE as i32,
                    );
                }

                FreeLibrary(hDevMgr);
            }

            LocalFree(lpDevInstID as HLOCAL);
        }
    }
}

unsafe extern "C" fn UpdateMonitorSelection(mut This: PDESKMONITOR) {
    let mut i: INT = 0;
    let mut lpDevInstID: LPTSTR = core::ptr::null_mut();
    if (*This).dwMonitorCount > 1 as DWORD {
        (*This).SelMonitor = core::ptr::null_mut();
        i = SendDlgItemMessage(
            (*This).hwndDlg,
            IDC_MONITORLIST as core::ffi::c_int,
            LB_GETCURSEL as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) as INT;
        if i >= 0 as INT {
            (*This).SelMonitor = SendDlgItemMessage(
                (*This).hwndDlg,
                IDC_MONITORLIST as core::ffi::c_int,
                LB_GETITEMDATA as UINT,
                i as WPARAM,
                0 as LPARAM,
            ) as PDESKMONINFO;
        }
    } else {
        (*This).SelMonitor = (*This).Monitors;
    }

    if !(*This).SelMonitor.is_null() {
        lpDevInstID = GetMonitorDevInstID(
            core::ptr::addr_of_mut!((*(*This).SelMonitor).dd.DeviceID) as *mut WCHAR as LPCTSTR,
        );
    }

    EnableWindow(
        GetDlgItem(
            (*This).hwndDlg,
            IDC_MONITORPROPERTIES as core::ffi::c_int,
        ),
        (!lpDevInstID.is_null()
            && *lpDevInstID.offset(0 as isize) != ('\0' as u16)) as BOOL,
    );

    if !lpDevInstID.is_null() {
        LocalFree(lpDevInstID as HLOCAL);
    }
}

unsafe extern "C" fn UpdatePruningControls(This: PDESKMONITOR) {
    EnableWindow(
        GetDlgItem((*This).hwndDlg, (IDC_PRUNINGCHECK as i32)),
        ((*This).__bindgen_anon_1.__bindgen_anon_1.bModesPruned() != 0 && (*This).__bindgen_anon_1.__bindgen_anon_1.bKeyIsReadOnly() == 0) as i32,
    );
    CheckDlgButton(
        (*This).hwndDlg,
        (IDC_PRUNINGCHECK as i32),
        (if (*This).__bindgen_anon_1.__bindgen_anon_1.bModesPruned() != 0 && (*This).__bindgen_anon_1.__bindgen_anon_1.bPruningOn() != 0 {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        }) as UINT,
    );
}

unsafe extern "C" fn GetPruningSettings(mut This: PDESKMONITOR) {
    let mut bModesPruned: BOOL = (FALSE as i32);
    let mut bKeyIsReadOnly: BOOL = (FALSE as i32);
    let mut bPruningOn: BOOL = (FALSE as i32);
    if !(*This).DeskExtInterface.is_null() {
        (*(*This).DeskExtInterface)
            .GetPruningMode
            .unwrap_unchecked()(
            (*(*This).DeskExtInterface).Context,
            &raw mut bModesPruned,
            &raw mut bKeyIsReadOnly,
            &raw mut bPruningOn,
        );
    }
    /* Check the boolean values against zero before assigning it to the bitfields! */
    (*This).__bindgen_anon_1.__bindgen_anon_1
        .set_bModesPruned((bModesPruned != (FALSE as i32)) as ::core::ffi::c_int as DWORD as DWORD);
    (*This).__bindgen_anon_1.__bindgen_anon_1
        .set_bKeyIsReadOnly((bKeyIsReadOnly != (FALSE as i32)) as ::core::ffi::c_int as DWORD as DWORD);
    (*This).__bindgen_anon_1.__bindgen_anon_1
        .set_bPruningOn((bPruningOn != (FALSE as i32)) as ::core::ffi::c_int as DWORD as DWORD);
    UpdatePruningControls(This);
}

unsafe extern "C" fn UpdateRefreshFrequencyList(mut This: PDESKMONITOR) {
    let mut lpCurrentMode: PDEVMODEW = core::ptr::null_mut();
    let mut lpMode: PDEVMODEW = core::ptr::null_mut();
    let mut szBuffer: [TCHAR; 64] = [0; 64];
    let mut dwIndex: DWORD = 0;
    let mut i: INT = 0;
    let mut bHasDef: BOOL = (FALSE as i32);
    let mut bAdded: BOOL = (FALSE as i32);

    /* Fill the refresh rate combo box */
    SendDlgItemMessage(
        (*This).hwndDlg,
        (IDC_REFRESHRATE as i32),
        CB_RESETCONTENT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );

    lpCurrentMode = (*(*This).DeskExtInterface)
        .GetCurrentMode
        .unwrap_unchecked()((*(*This).DeskExtInterface).Context);
    dwIndex = 0 as DWORD;

    loop {
        'body: {
            let fresh15 = dwIndex;
            dwIndex = dwIndex.wrapping_add(1);
            lpMode = (*(*This).DeskExtInterface)
                .EnumAllModes
                .unwrap_unchecked()(
                (*(*This).DeskExtInterface).Context,
                fresh15,
            );
            if !lpMode.is_null()
                && (*lpMode).dmBitsPerPel == (*lpCurrentMode).dmBitsPerPel
                && (*lpMode).dmPelsWidth == (*lpCurrentMode).dmPelsWidth
                && (*lpMode).dmPelsHeight == (*lpCurrentMode).dmPelsHeight
            {
                /* We're only interested in refresh rates for the current resolution and color depth */

                if (*lpMode).dmDisplayFrequency <= 1 as DWORD {
                    /* Default hardware frequency */
                    if bHasDef != 0 {
                        break 'body;
                    }

                    bHasDef = (TRUE as i32);

                    if LoadString(
                        hInstance,
                        IDS_USEDEFFRQUENCY as UINT,
                        core::ptr::addr_of_mut!(szBuffer) as LPTSTR,
                        (core::mem::size_of::<[TCHAR; 64]>()
                            / core::mem::size_of_val(&szBuffer[0]))
                            as INT,
                    ) == 0
                    {
                        szBuffer[0] = ('\0' as u16);
                    }
                } else {
                    let mut szFmt: [TCHAR; 64] = [0; 64];

                    if LoadString(
                        hInstance,
                        IDS_FREQFMT as UINT,
                        core::ptr::addr_of_mut!(szFmt) as LPTSTR,
                        (core::mem::size_of::<[TCHAR; 64]>()
                            / core::mem::size_of_val(&szFmt[0]))
                            as INT,
                    ) == 0
                    {
                        szFmt[0] = ('\0' as u16);
                    }

                    _sntprintf(
                        core::ptr::addr_of_mut!(szBuffer) as *mut TCHAR,
                        core::mem::size_of::<[TCHAR; 64]>()
                            / core::mem::size_of_val(&szBuffer[0]),
                        core::ptr::addr_of_mut!(szFmt) as *mut TCHAR,
                        (*lpMode).dmDisplayFrequency,
                    );
                }

                i = SendDlgItemMessage(
                    (*This).hwndDlg,
                    (IDC_REFRESHRATE as i32),
                    CB_ADDSTRING as UINT,
                    0 as WPARAM,
                    core::ptr::addr_of_mut!(szBuffer) as *mut TCHAR as LPARAM,
                ) as INT;
                if i >= 0 {
                    bAdded = (TRUE as i32);

                    SendDlgItemMessage(
                        (*This).hwndDlg,
                        (IDC_REFRESHRATE as i32),
                        CB_SETITEMDATA as UINT,
                        i as WPARAM,
                        lpMode as LPARAM,
                    );

                    if (*lpMode).dmDisplayFrequency == (*lpCurrentMode).dmDisplayFrequency {
                        SendDlgItemMessage(
                            (*This).hwndDlg,
                            (IDC_REFRESHRATE as i32),
                            CB_SETCURSEL as UINT,
                            i as WPARAM,
                            0 as LPARAM,
                        );
                    }
                }
            }
        }

        if lpMode.is_null() {
            break;
        }
    }

    EnableWindow(
        GetDlgItem((*This).hwndDlg, (IDS_MONITORSETTINGSGROUP as i32)),
        bAdded,
    );
    EnableWindow(
        GetDlgItem((*This).hwndDlg, (IDS_REFRESHRATELABEL as i32)),
        bAdded,
    );
    EnableWindow(
        GetDlgItem((*This).hwndDlg, (IDC_REFRESHRATE as i32)),
        bAdded,
    );

    GetPruningSettings(This);
}

unsafe extern "C" fn InitMonitorDialog(mut This: PDESKMONITOR) {
    let mut pmi: PDESKMONINFO = core::ptr::null_mut();
    let mut pminext: PDESKMONINFO = core::ptr::null_mut();
    let mut pmilink: *mut PDESKMONINFO = core::ptr::null_mut();
    let mut lpDeviceId: LPTSTR = core::ptr::null_mut();
    let mut cr: CONFIGRET = 0;
    let mut dd: DISPLAY_DEVICE = core::mem::zeroed();
    let mut bRet: BOOL = 0;
    let mut i: INT = 0;
    let mut dwIndex: DWORD = 0;

    /* Free all allocated monitors */
    pmi = (*This).Monitors;
    (*This).Monitors = core::ptr::null_mut();
    while !pmi.is_null() {
        pminext = (*pmi).Next;
        LocalFree(pmi as HLOCAL);
        pmi = pminext;
    }

    (*This).SelMonitor = core::ptr::null_mut();
    (*This).dwMonitorCount = 0;

    bFoundAdapter = (FALSE as i32);
    lpDeviceId = QueryDeskCplString(
        (*This).pdtobj,
        RegisterClipboardFormat(DESK_EXT_DISPLAYID),
    );

    if !lpDeviceId.is_null() && *lpDeviceId.offset(0) != ('\0' as u16) {
        cr = CM_Locate_DevNode(
            &raw mut diAdapter,
            lpDeviceId,
            CM_LOCATE_DEVNODE_NORMAL,
        );
        bFoundAdapter = (cr == CR_SUCCESS) as BOOL;

        if bFoundAdapter == 0 {
            DPRINT1!("CM_Locate_DevNode failed: %d\n", cr);
        }
    }

    if !(*This).lpDisplayDevice.is_null() {
        LocalFree((*This).lpDisplayDevice as HLOCAL);
    }

    (*This).lpDisplayDevice = QueryDeskCplString(
        (*This).pdtobj,
        RegisterClipboardFormat(DESK_EXT_DISPLAYDEVICE),
    );

    if !(*This).DeskExtInterface.is_null() {
        if !(*This).lpDisplayDevice.is_null() {
            /* Enumerate all monitors */
            dwIndex = 0;
            pmilink = &raw mut (*This).Monitors;

            loop {
                dd.cb = core::mem::size_of::<DISPLAY_DEVICE>() as DWORD;
                let fresh16 = dwIndex;
                dwIndex = dwIndex.wrapping_add(1);
                bRet = EnumDisplayDevices(
                    (*This).lpDisplayDevice,
                    fresh16,
                    &raw mut dd,
                    0,
                );
                if bRet != 0 {
                    pmi = LocalAlloc(
                        LMEM_FIXED,
                        core::mem::size_of::<_DESKMONINFO>() as SIZE_T,
                    ) as PDESKMONINFO;
                    if !pmi.is_null() {
                        CopyMemory!(
                            &raw mut (*pmi).dd as *mut core::ffi::c_void,
                            &raw mut dd as *const core::ffi::c_void,
                            core::mem::size_of::<DISPLAY_DEVICE>() as usize,
                        );
                        (*pmi).Next = core::ptr::null_mut();
                        *pmilink = pmi;
                        pmilink = &raw mut (*pmi).Next;

                        (*This).dwMonitorCount = (*This).dwMonitorCount.wrapping_add(1);
                    }
                }
                if !(bRet != 0) {
                    break;
                }
            }
        }

        (*This).lpDevModeOnInit = (*(*This).DeskExtInterface)
            .GetCurrentMode
            .unwrap_unchecked()(
            (*(*This).DeskExtInterface).Context
        );
    } else {
        (*This).lpDevModeOnInit = core::ptr::null_mut();
    }

    (*This).lpSelDevMode = (*This).lpDevModeOnInit;

    /* Setup the UI depending on how many monitors are attached */
    if (*This).dwMonitorCount == 0 {
        let mut lpMonitorName: LPTSTR = core::ptr::null_mut();

        /* This is a fallback, let's hope that desk.cpl can provide us with a monitor name */
        lpMonitorName = QueryDeskCplString(
            (*This).pdtobj,
            RegisterClipboardFormat(DESK_EXT_MONITORNAME),
        );

        SetDlgItemText(
            (*This).hwndDlg,
            (IDC_MONITORNAME as i32),
            lpMonitorName,
        );

        if !lpMonitorName.is_null() {
            LocalFree(lpMonitorName as HLOCAL);
        }
    } else if (*This).dwMonitorCount == 1 {
        (*This).SelMonitor = (*This).Monitors;
        SetDlgItemText(
            (*This).hwndDlg,
            (IDC_MONITORNAME as i32),
            &raw mut (*(*This).Monitors).dd.DeviceString as *mut WCHAR as LPCTSTR,
        );
    } else {
        SendDlgItemMessage(
            (*This).hwndDlg,
            (IDC_MONITORLIST as i32),
            LB_RESETCONTENT,
            0,
            0,
        );

        pmi = (*This).Monitors;
        while !pmi.is_null() {
            i = SendDlgItemMessage(
                (*This).hwndDlg,
                (IDC_MONITORLIST as i32),
                LB_ADDSTRING,
                0,
                &raw mut (*pmi).dd.DeviceString as *mut WCHAR as LPARAM,
            ) as INT;
            if i >= 0 {
                SendDlgItemMessage(
                    (*This).hwndDlg,
                    (IDC_MONITORLIST as i32),
                    LB_SETITEMDATA,
                    i as WPARAM,
                    pmi as LPARAM,
                );

                if (*This).SelMonitor.is_null() {
                    SendDlgItemMessage(
                        (*This).hwndDlg,
                        (IDC_MONITORLIST as i32),
                        LB_SETCURSEL,
                        i as WPARAM,
                        0,
                    );

                    (*This).SelMonitor = pmi;
                }
            }

            pmi = (*pmi).Next;
        }
    }

    /* Show/Hide controls */
    ShowWindow(
        GetDlgItem((*This).hwndDlg, (IDC_MONITORNAME as i32)),
        if (*This).dwMonitorCount <= 1 {
            (SW_SHOW as i32)
        } else {
            (SW_HIDE as i32)
        },
    );
    ShowWindow(
        GetDlgItem((*This).hwndDlg, (IDC_MONITORLIST as i32)),
        if (*This).dwMonitorCount > 1 {
            (SW_SHOW as i32)
        } else {
            (SW_HIDE as i32)
        },
    );

    UpdateRefreshFrequencyList(This);
    UpdateMonitorSelection(This);
}

unsafe extern "C" fn UpdatePruningSelection(mut This: PDESKMONITOR) {
    let mut bPruningOn: BOOL = 0;
    if !(*This).DeskExtInterface.is_null()
        && (*This).__bindgen_anon_1.__bindgen_anon_1.bModesPruned() as ::core::ffi::c_int != 0
        && (*This).__bindgen_anon_1.__bindgen_anon_1.bKeyIsReadOnly() == 0
    {
        bPruningOn = (IsDlgButtonChecked((*This).hwndDlg, IDC_PRUNINGCHECK as core::ffi::c_int)
            != BST_UNCHECKED as UINT) as ::core::ffi::c_int as BOOL;
        if bPruningOn != (*This).__bindgen_anon_1.__bindgen_anon_1.bPruningOn() as ::core::ffi::c_int {
            (*This).__bindgen_anon_1.__bindgen_anon_1
                .set_bPruningOn(bPruningOn as DWORD as DWORD);
            (*(*This).DeskExtInterface)
                .SetPruningMode
                .unwrap_unchecked()(
                (*(*This).DeskExtInterface).Context,
                bPruningOn,
            );
            UpdateRefreshFrequencyList(This);
            let _ = PropSheet_Changed!(
                GetParent((*This).hwndDlg),
                (*This).hwndDlg,
            );
        }
    }
}

unsafe extern "C" fn UpdateRefreshRateSelection(mut This: PDESKMONITOR) {
    let mut lpCurrentDevMode: PDEVMODEW = ::core::ptr::null_mut::<_devicemodeW>();
    let mut i: INT = 0;
    if !(*This).DeskExtInterface.is_null() {
        i = SendDlgItemMessage(
            (*This).hwndDlg,
            (IDC_REFRESHRATE as i32),
            CB_GETCURSEL,
            0,
            0,
        ) as INT;
        if i >= 0 {
            lpCurrentDevMode = (*This).lpSelDevMode;
            (*This).lpSelDevMode = SendDlgItemMessage(
                (*This).hwndDlg,
                (IDC_REFRESHRATE as i32),
                CB_GETITEMDATA,
                i as WPARAM,
                0,
            ) as PDEVMODEW;

            if !(*This).lpSelDevMode.is_null() && (*This).lpSelDevMode != lpCurrentDevMode {
                (*(*This).DeskExtInterface)
                    .SetCurrentMode
                    .unwrap_unchecked()(
                    (*(*This).DeskExtInterface).Context,
                    (*This).lpSelDevMode as *mut DEVMODEW,
                );

                UpdateRefreshFrequencyList(This);

                let _ = PropSheet_Changed!(
                    GetParent((*This).hwndDlg),
                    (*This).hwndDlg,
                );
            }
        }
    }
}

unsafe extern "C" fn ApplyMonitorChanges(mut This: PDESKMONITOR) -> LONG {
    let mut lChangeRet: LONG = 0;
    if !(*This).DeskExtInterface.is_null() {
        /* Change the display settings through desk.cpl */
        lChangeRet = DeskCplExtDisplaySaveSettings((*This).DeskExtInterface, (*This).hwndDlg);
        if lChangeRet == DISP_CHANGE_SUCCESSFUL as LONG {
            /* Save the new mode */
            (*This).lpDevModeOnInit = (*(*This).DeskExtInterface)
                .GetCurrentMode
                .unwrap_unchecked()(
                (*(*This).DeskExtInterface).Context
            );
            (*This).lpSelDevMode = (*This).lpDevModeOnInit;
            return PSNRET_NOERROR as LONG;
        } else if lChangeRet == DISP_CHANGE_RESTART as LONG {
            /* Notify desk.cpl that the user needs to reboot */
            PropSheet_RestartWindows!(GetParent((*This).hwndDlg));
            return PSNRET_NOERROR as LONG;
        }
    }

    InitMonitorDialog(This);

    return PSNRET_INVALID_NOCHANGEPAGE as LONG;
}

unsafe extern "C" fn ResetMonitorChanges(This: PDESKMONITOR) {
    if !(*This).DeskExtInterface.is_null() && !(*This).lpDevModeOnInit.is_null() {
        (*(*This).DeskExtInterface)
            .SetCurrentMode
            .unwrap_unchecked()(
            (*(*This).DeskExtInterface).Context,
            (*This).lpDevModeOnInit as *mut DEVMODEW,
        );
    }
}

unsafe extern "system" fn MonitorDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut This: PDESKMONITOR = core::ptr::null_mut();
    let mut Ret: INT_PTR = 0;

    if uMsg != WM_INITDIALOG {
        This = GetWindowLongPtr(hwndDlg, DWLP_USER) as PDESKMONITOR;
    }

    match uMsg {
        WM_INITDIALOG => {
            This = (*(lParam as LPCPROPSHEETPAGE)).lParam as PDESKMONITOR;
            (*This).hwndDlg = hwndDlg;
            SetWindowLongPtr(hwndDlg, DWLP_USER, This as LONG_PTR);

            InitMonitorDialog(This);
            Ret = TRUE as INT_PTR;
        }

        WM_COMMAND => {
            match ((LOWORD!(wParam)) as ::core::ffi::c_int) as u32 {
                IDC_MONITORPROPERTIES => {
                    ShowMonitorProperties(This);
                }

                IDC_MONITORLIST => {
                    if (HIWORD!(wParam)) as ::core::ffi::c_int == (LBN_SELCHANGE as i32) {
                        UpdateMonitorSelection(This);
                    }
                }

                IDC_PRUNINGCHECK => {
                    if (HIWORD!(wParam)) as ::core::ffi::c_int == (BN_CLICKED as i32) {
                        UpdatePruningSelection(This);
                    }
                }

                IDC_REFRESHRATE => {
                    if (HIWORD!(wParam)) as ::core::ffi::c_int == (CBN_SELCHANGE as i32) {
                        UpdateRefreshRateSelection(This);
                    }
                }

                _ => {}
            }
        }

        WM_NOTIFY => {
            let nmh: *mut NMHDR = lParam as *mut NMHDR;

            match ((*nmh).code) as i32 {
                PSN_APPLY => {
                    SetWindowLongPtr(
                        hwndDlg,
                        DWLP_MSGRESULT,
                        ApplyMonitorChanges(This) as LONG_PTR,
                    );
                    Ret = TRUE as INT_PTR;
                }

                PSN_RESET => {
                    ResetMonitorChanges(This);
                }

                PSN_SETACTIVE => {
                    UpdateRefreshFrequencyList(This);
                }

                _ => {}
            }
        }

        _ => {}
    }

    return Ret;
}

unsafe extern "C" fn IDeskMonitor_Destroy(This: PDESKMONITOR) {
    let mut pmi: PDESKMONINFO;
    let mut pminext: PDESKMONINFO;

    if !(*This).pdtobj.is_null() {
        IDataObject_Release((*This).pdtobj);
        (*This).pdtobj = core::ptr::null_mut();
    }

    if !(*This).DeskExtInterface.is_null() {
        LocalFree((*This).DeskExtInterface as HLOCAL);
        (*This).DeskExtInterface = core::ptr::null_mut();
    }

    if !(*This).lpDisplayDevice.is_null() {
        LocalFree((*This).lpDisplayDevice as HLOCAL);
        (*This).lpDisplayDevice = core::ptr::null_mut();
    }

    /* Free all monitors */
    pmi = (*This).Monitors;
    (*This).Monitors = core::ptr::null_mut();
    while !pmi.is_null() {
        pminext = (*pmi).Next;
        LocalFree(pmi as HLOCAL);
        pmi = pminext;
    }
}

#[no_mangle]
pub unsafe extern "C" fn IDeskMonitor_AddRef(This: PDESKMONITOR) -> ULONG {
    let mut ret: ULONG;
    ret = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as PLONG) as ULONG;
    if ret == 1 {
        InterlockedIncrement(core::ptr::addr_of_mut!(dll_refs));
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn IDeskMonitor_Release(This: PDESKMONITOR) -> ULONG {
    let mut ret: ULONG;

    ret = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as PLONG) as ULONG;
    if ret == 0 {
        IDeskMonitor_Destroy(This);
        InterlockedDecrement(core::ptr::addr_of_mut!(dll_refs) as PLONG);

        HeapFree(GetProcessHeap(),
                 0,
                 This as PVOID);
    }

    return ret;
}
#[no_mangle]
pub unsafe extern "system" fn IDeskMonitor_QueryInterface(This: PDESKMONITOR, iid: *const IID, pvObject: *mut PVOID) -> HRESULT {
    *pvObject = core::ptr::null_mut();

    if IsEqualIID!(iid, core::ptr::addr_of!(IID_IShellPropSheetExt)) != 0
        || IsEqualIID!(iid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *pvObject = impl_to_interface!(This, IShellPropSheetExt) as PVOID;
    } else if IsEqualIID!(iid, core::ptr::addr_of!(IID_IShellExtInit)) != 0 {
        *pvObject = impl_to_interface!(This, IShellExtInit) as PVOID;
    } else if IsEqualIID!(iid, core::ptr::addr_of!(IID_IClassFactory)) != 0 {
        *pvObject = impl_to_interface!(This, IClassFactory) as PVOID;
    } else {
        DPRINT1!("IDeskMonitor::QueryInterface(%p,%p): E_NOINTERFACE\n", iid, pvObject);
        return E_NOINTERFACE;
    }

    IDeskMonitor_AddRef(This);
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn IDeskMonitor_Initialize(This: PDESKMONITOR, pidlFolder: LPCITEMIDLIST, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    DPRINT1!("IDeskMonitor::Initialize(%p,%p,%p)\n", pidlFolder, pdtobj, hkeyProgID);

    if !pdtobj.is_null() {
        IDataObject_AddRef(pdtobj);
        (*This).pdtobj = pdtobj;

        /* Get a copy of the desk.cpl extension interface */
        (*This).DeskExtInterface = QueryDeskCplExtInterface((*This).pdtobj);
        if !(*This).DeskExtInterface.is_null() {
            return S_OK;
        }
    }

    return S_FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn IDeskMonitor_AddPages(mut This: PDESKMONITOR, mut pfnAddPage: LPFNADDPROPSHEETPAGE, mut lParam: LPARAM) -> HRESULT {
    let mut hpsp: HPROPSHEETPAGE = core::ptr::null_mut();
    let mut psp: PROPSHEETPAGE = core::mem::zeroed();

    DPRINT1!("IDeskMonitor::AddPages(%p,%p)\n", pfnAddPage, lParam);

    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    psp.dwFlags = PSP_DEFAULT as DWORD;
    psp.hInstance = hInstance;
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_MONITOR);
    psp.pfnDlgProc = Some(MonitorDlgProc);
    psp.lParam = This as LPARAM;

    hpsp = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));
    if !hpsp.is_null()
        && pfnAddPage.unwrap_unchecked()(hpsp, lParam) != 0
    {
        return S_OK as HRESULT;
    }

    return S_FALSE as HRESULT;
}

#[no_mangle]
pub unsafe extern "C" fn IDeskMonitor_ReplacePage(This: PDESKMONITOR, uPageID: EXPPS, pfnReplacePage: LPFNADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
    DPRINT1!("IDeskMonitor::ReplacePage(%u,%p,%p)\n", uPageID, pfnReplacePage, lParam);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IDeskMonitor_Constructor(riid: *const IID, mut ppv: *mut LPVOID) -> HRESULT {
    let mut This: PDESKMONITOR;
    let mut hRet: HRESULT = E_OUTOFMEMORY;

    DPRINT1!("IDeskMonitor::Constructor(%p,%p)\n", riid, ppv);

    This = HeapAlloc(
        GetProcessHeap(),
        0,
        ::core::mem::size_of::<_DESKMONITOR>() as SIZE_T,
    ) as PDESKMONITOR;
    if !This.is_null() {
        ZeroMemory!(
            This as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<_DESKMONITOR>() as SIZE_T,
        );

        IDeskMonitor_InitIface(This);

        hRet = IDeskMonitor_QueryInterface(This, riid, ppv as *mut PVOID);
        if !(hRet >= 0) {
            IDeskMonitor_Release(This);
        }
    }

    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, dwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            hInstance = hinstDLL;
            DisableThreadLibraryCalls(hInstance as HMODULE);
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
