use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_1DE543427BD7: [WCHAR; 51] = [0x0054, 0x0068, 0x0065, 0x0020, 0x0025, 0x0077, 0x0053, 0x0020, 0x0063, 0x0061, 0x006E, 0x0020, 0x006E, 0x006F, 0x0077, 0x0020, 0x0062, 0x0065, 0x0020, 0x0073, 0x0061, 0x0066, 0x0065, 0x006C, 0x0079, 0x0020, 0x0072, 0x0065, 0x006D, 0x006F, 0x0076, 0x0065, 0x0064, 0x0020, 0x0066, 0x0072, 0x006F, 0x006D, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x002E, 0x0000];
pub static CPP_COM_WSTR_4FCF19ECA92F: [WCHAR; 69] = [0x0054, 0x0068, 0x0065, 0x0020, 0x0064, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0063, 0x0061, 0x006E, 0x006E, 0x006F, 0x0074, 0x0020, 0x0062, 0x0065, 0x0020, 0x0073, 0x0074, 0x006F, 0x0070, 0x0070, 0x0065, 0x0064, 0x0020, 0x0072, 0x0069, 0x0067, 0x0068, 0x0074, 0x0020, 0x006E, 0x006F, 0x0077, 0x0021, 0x0020, 0x0054, 0x0072, 0x0079, 0x0020, 0x0073, 0x0074, 0x006F, 0x0070, 0x0070, 0x0069, 0x006E, 0x0067, 0x0020, 0x0069, 0x0074, 0x0020, 0x0061, 0x0067, 0x0061, 0x0069, 0x006E, 0x0020, 0x006C, 0x0061, 0x0074, 0x0065, 0x0072, 0x0021, 0x0000];
pub static CPP_COM_WSTR_76F513E8A257: [WCHAR; 15] = [0x0055, 0x006E, 0x006B, 0x006E, 0x006F, 0x0077, 0x006E, 0x0020, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0000];
pub static CPP_COM_WSTR_8423327A37F3: [WCHAR; 21] = [0x0050, 0x0072, 0x006F, 0x0062, 0x006C, 0x0065, 0x006D, 0x0020, 0x0045, 0x006A, 0x0065, 0x0063, 0x0074, 0x0069, 0x006E, 0x0067, 0x0020, 0x0025, 0x0077, 0x0053, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_B87E4F458B30: [WCHAR; 24] = [0x0053, 0x0061, 0x0066, 0x0065, 0x0020, 0x0074, 0x006F, 0x0020, 0x0052, 0x0065, 0x006D, 0x006F, 0x0076, 0x0065, 0x0020, 0x0048, 0x0061, 0x0072, 0x0064, 0x0077, 0x0061, 0x0072, 0x0065, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut g_devList: CSimpleArray<DEVINST> = unsafe { core::mem::zeroed() };
pub static mut g_strHotplugTooltip: CString = unsafe { core::mem::zeroed() };
pub static mut g_hIconHotplug: HICON = core::ptr::null_mut();
pub static mut g_strMenuSel: [WCHAR; 40] = [0; 40];
pub static mut g_IsRemoving: BOOL = 0;
#[no_mangle]
pub unsafe extern "C" fn EnumHotpluggedDevices(devList: *mut CSimpleArray<DEVINST>) -> HRESULT {
    (*devList).RemoveAll(); // Clear current devList
    let hdev: HDEVINFO = SetupDiGetClassDevs(core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), DIGCF_ALLCLASSES | DIGCF_PRESENT);
    if INVALID_HANDLE_VALUE == hdev {
        return E_HANDLE;
    }
    let mut did: SP_DEVINFO_DATA = core::mem::zeroed();
    did.cbSize = core::mem::size_of::<SP_DEVINFO_DATA>() as u32;

    // Enumerate all the attached devices.
    let mut idev: i32 = 0;
    while SetupDiEnumDeviceInfo(hdev, (idev) as DWORD, core::ptr::addr_of_mut!(did)) != 0 {
        'body: {
            let mut dwCapabilities: DWORD = 0;
            let mut dwSize: DWORD = core::mem::size_of::<DWORD>() as DWORD;
            let mut ulStatus: ULONG = 0;
            let mut ulProblem: ULONG = 0;
            let mut cr: CONFIGRET = CM_Get_DevNode_Status(core::ptr::addr_of_mut!(ulStatus), core::ptr::addr_of_mut!(ulProblem), did.DevInst, 0);
            if cr != CR_SUCCESS {
                break 'body;
            }

            cr = CM_Get_DevNode_Registry_Property(did.DevInst, CM_DRP_CAPABILITIES, core::ptr::null_mut(), (core::ptr::addr_of_mut!(dwCapabilities)).cast(), core::ptr::addr_of_mut!(dwSize), 0);
            if cr != CR_SUCCESS {
                break 'body;
            }

            // Filter and make list of only the appropriate safely removable devices.
            if (dwCapabilities & CM_DEVCAP_REMOVABLE) != 0 &&
                (dwCapabilities & CM_DEVCAP_DOCKDEVICE) == 0 &&
                (dwCapabilities & CM_DEVCAP_SURPRISEREMOVALOK) == 0 &&
                ((dwCapabilities & CM_DEVCAP_EJECTSUPPORTED) != 0 || (ulStatus & DN_DISABLEABLE) != 0) &&
                ulProblem == 0
            {
                (*devList).Add(did.DevInst);
            }
        }
        idev += 1;
    }
    SetupDiDestroyDeviceInfoList(hdev);

    if NO_ERROR != GetLastError() && ERROR_NO_MORE_ITEMS != GetLastError() {
        return E_UNEXPECTED;
    }

    S_OK
}
#[no_mangle]
pub unsafe extern "C" fn NotifyBalloon(pSysTray: *mut CSysTray, szTitle: LPCWSTR, szInfo: LPCWSTR, uId: UINT) -> HRESULT {
    let mut nim: NOTIFYICONDATA = core::mem::zeroed();

    nim.cbSize = core::mem::size_of::<NOTIFYICONDATA>() as DWORD;
    nim.uID = uId;
    nim.hWnd = (*pSysTray).GetHWnd();

    nim.uFlags = NIF_INFO;
    nim.__bindgen_anon_1.uTimeout = 10;
    nim.dwInfoFlags = NIIF_INFO;

    StringCchCopy(core::ptr::addr_of_mut!(nim.szInfoTitle).cast(), (_countof!({ nim.szInfoTitle })) as _, szTitle);
    StringCchCopy(core::ptr::addr_of_mut!(nim.szInfo).cast(), (_countof!({ nim.szInfo })) as _, szInfo);
    let mut ret: BOOL = Shell_NotifyIcon(NIM_MODIFY, core::ptr::addr_of_mut!(nim));

    Sleep(10000); /* As per windows, the balloon notification remains visible for atleast 10 sec.
                     This timer maintains the same condition.
                     Also it is required so that the icon doesn't hide instantly after last device is removed,
                     as that will prevent popping of notification.
                  */
    StringCchCopy(core::ptr::addr_of_mut!(nim.szInfoTitle).cast(), (_countof!({ nim.szInfoTitle })) as _, CPP_COM_WSTR_96A296D224F2.as_ptr());
    StringCchCopy(core::ptr::addr_of_mut!(nim.szInfo).cast(), (_countof!({ nim.szInfo })) as _, CPP_COM_WSTR_96A296D224F2.as_ptr());
    ret = Shell_NotifyIcon(NIM_MODIFY, core::ptr::addr_of_mut!(nim));
    g_IsRemoving = (FALSE as i32); /* This flag is used to prevent instant icon hiding after last device is removed.
                             The above timer maintains the required state for the same.
                          */
    return if ret != 0 { S_OK } else { E_FAIL };
}
#[no_mangle]
pub unsafe extern "system" fn Hotplug_Init(pSysTray: *mut CSysTray) -> HRESULT {
    g_hIconHotplug = LoadIcon(g_hInstance, MAKEINTRESOURCE!(IDI_HOTPLUG_OK));
    (*core::ptr::addr_of_mut!(g_strHotplugTooltip)).LoadStringW(IDS_HOTPLUG_REMOVE_1);

    EnumHotpluggedDevices(&mut *core::ptr::addr_of_mut!(g_devList));

    if (*core::ptr::addr_of_mut!(g_devList)).GetSize() > 0 {
        return (*pSysTray).NotifyIcon((NIM_ADD) as INT, ID_ICON_HOTPLUG, g_hIconHotplug, (*core::ptr::addr_of!(g_strHotplugTooltip)).GetString(), 0);
    } else {
        return (*pSysTray).NotifyIcon((NIM_ADD) as INT, ID_ICON_HOTPLUG, g_hIconHotplug, (*core::ptr::addr_of!(g_strHotplugTooltip)).GetString(), NIS_HIDDEN);
    }
}

#[no_mangle]
pub unsafe extern "system" fn Hotplug_Update(pSysTray: *mut CSysTray) -> HRESULT {
    // TRACE("Hotplug_Update\n");
    S_OK
}
#[no_mangle]
pub unsafe extern "system" fn Hotplug_Shutdown(pSysTray: *mut CSysTray) -> HRESULT {
    DestroyIcon(g_hIconHotplug);
    g_hIconHotplug = core::ptr::null_mut();

    return (*pSysTray).NotifyIcon((NIM_DELETE) as INT, ID_ICON_HOTPLUG, core::ptr::null_mut(), core::ptr::null_mut(), 0);
}

unsafe extern "C" fn _RunHotplug(pSysTray: *mut CSysTray) {
    CSysTray::RunDll(b"hotplug.dll\0".as_ptr() as *const i8, b"\0".as_ptr() as *const i8);
}

pub unsafe extern "C" fn _ShowContextMenu(pSysTray: *mut CSysTray) {
    let mut hPopup: HMENU = CreatePopupMenu();
    let mut ulLength: ULONG = DISPLAY_NAME_LEN as ULONG * core::mem::size_of::<WCHAR>() as ULONG;

    let mut index: INT = 0;
    while index < g_devList.GetSize() {
        let mut dispName: [WCHAR; (DISPLAY_NAME_LEN as _)] = [0 as WCHAR; (DISPLAY_NAME_LEN as _)];
        let mut menuName = CStringW::new();
        let mut cr: CONFIGRET = CM_Get_DevNode_Registry_Property(
            g_devList[(index) as usize],
            CM_DRP_DEVICEDESC,
            core::ptr::null_mut(),
            (dispName.as_mut_ptr()).cast(),
            core::ptr::addr_of_mut!(ulLength),
            0,
        );
        if cr != CR_SUCCESS {
            StrCpyW(dispName.as_mut_ptr(), CPP_COM_WSTR_76F513E8A257.as_ptr());
        }

        atl_format_id!(menuName, IDS_HOTPLUG_REMOVE_3, dispName.as_ptr());
        AppendMenuW(
            hPopup,
            MF_STRING,
            (index + 1) as UINT_PTR,
            menuName.GetString(),
        );
        index += 1;
    }

    SetForegroundWindow((*pSysTray).GetHWnd());
    let mut flags: DWORD = TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTALIGN | TPM_BOTTOMALIGN;
    let mut pt: POINT = core::mem::zeroed();
    GetCursorPos(core::ptr::addr_of_mut!(pt));

    let mut id: DWORD = (TrackPopupMenuEx(
        hPopup,
        flags,
        pt.x,
        pt.y,
        (*pSysTray).GetHWnd(),
        core::ptr::null_mut(),
    )) as u32;

    if id > 0 {
        id -= 1; // since array indices starts from zero.
        let mut cr: CONFIGRET = CM_Get_DevNode_Registry_Property(
            g_devList[(id) as _],
            CM_DRP_DEVICEDESC,
            core::ptr::null_mut(),
            (core::ptr::addr_of_mut!(g_strMenuSel) as *mut WCHAR).cast(),
            core::ptr::addr_of_mut!(ulLength),
            0,
        );
        if cr != CR_SUCCESS {
            StrCpyW(
                core::ptr::addr_of_mut!(g_strMenuSel) as *mut WCHAR,
                CPP_COM_WSTR_76F513E8A257.as_ptr(),
            );
        }

        cr = CM_Request_Device_Eject_Ex(
            g_devList[(id) as _],
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            0,
            core::ptr::null_mut(),
        );
        if cr != CR_SUCCESS {
            let mut strInfo: [WCHAR; 128] = [0 as WCHAR; 128];
            _swprintf(
                strInfo.as_mut_ptr(),
                CPP_COM_WSTR_8423327A37F3.as_ptr(),
                core::ptr::addr_of_mut!(g_strMenuSel) as *mut WCHAR,
            );
            MessageBox(
                core::ptr::null_mut(),
                CPP_COM_WSTR_4FCF19ECA92F.as_ptr(),
                strInfo.as_mut_ptr(),
                MB_OK | MB_ICONEXCLAMATION,
            );
        } else {
            //MessageBox(0, L"Device ejected successfully!! You can safely remove the device now!", L"Safely Remove Hardware", MB_OK | MB_ICONINFORMATION);
            g_IsRemoving = (TRUE as i32);
            g_devList.RemoveAt((id) as i32); /* thing is.. even after removing id at this point, the devnode_change occurs after some seconds of sucessful removal
                                       and since pendrive is still plugged in it gets enumerated, if problem number is not filtered.
                                    */
        }
    }

    DestroyMenu(hPopup);
}

unsafe extern "C" fn _ShowContextMenuR(pSysTray: *mut CSysTray) {
    let strMenu = CStringW::from_ptr(IDS_HOTPLUG_REMOVE_2 as LPWSTR);
    let hPopup: HMENU = CreatePopupMenu();
    AppendMenuW(hPopup, MF_STRING, (IDS_HOTPLUG_REMOVE_2) as UINT_PTR, strMenu.GetString());
    SetMenuDefaultItem(hPopup, IDS_HOTPLUG_REMOVE_2, FALSE);

    SetForegroundWindow((*pSysTray).GetHWnd());
    let flags: DWORD = TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTALIGN | TPM_BOTTOMALIGN;
    let mut pt: POINT = core::mem::zeroed();
    GetCursorPos(core::ptr::addr_of_mut!(pt));

    let id: DWORD = (TrackPopupMenuEx(hPopup, flags,
        pt.x, pt.y,
        (*pSysTray).GetHWnd(), core::ptr::null_mut())) as u32;

    if id == IDS_HOTPLUG_REMOVE_2
    {
        _RunHotplug(pSysTray);
    }

    DestroyMenu(hPopup);
}
#[no_mangle]
pub unsafe extern "C" fn HotplugDeviceTimer(pSysTray: *mut CSysTray) {
    EnumHotpluggedDevices(&mut *core::ptr::addr_of_mut!(g_devList));

    if g_devList.GetSize() > 0 {
        (*pSysTray).NotifyIcon((NIM_MODIFY) as INT, ID_ICON_HOTPLUG, g_hIconHotplug, g_strHotplugTooltip.GetString(), 0);
    } else {
        (*pSysTray).NotifyIcon((NIM_MODIFY) as INT, ID_ICON_HOTPLUG, g_hIconHotplug, g_strHotplugTooltip.GetString(), NIS_HIDDEN);
    }
}
#[no_mangle]
pub unsafe extern "system" fn Hotplug_Message(pSysTray: *mut CSysTray, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, lResult: &mut LRESULT) -> HRESULT {
    match uMsg {
        v if v == WM_USER + 220 => {
            TRACE!("Hotplug_Message: WM_USER+220\n");
            if wParam == (HOTPLUG_SERVICE_FLAG as _) {
                if lParam != 0 {
                    (*pSysTray).EnableService(HOTPLUG_SERVICE_FLAG, (TRUE) as BOOL);
                    return Hotplug_Init(pSysTray);
                } else {
                    (*pSysTray).EnableService(HOTPLUG_SERVICE_FLAG, (FALSE) as BOOL);
                    return Hotplug_Shutdown(pSysTray);
                }
            }
            return S_FALSE;
        }

        v if v == WM_USER + 221 => {
            TRACE!("Hotplug_Message: WM_USER+221\n");
            if wParam == (HOTPLUG_SERVICE_FLAG as _) {
                *lResult = (*pSysTray).IsServiceEnabled(HOTPLUG_SERVICE_FLAG) as LRESULT;
                return S_OK;
            }
            return S_FALSE;
        }

        v if v == WM_TIMER => {
            if wParam == (HOTPLUG_TIMER_ID as _) {
                KillTimer((*pSysTray).GetHWnd(), (HOTPLUG_TIMER_ID as _));
                _ShowContextMenu(pSysTray);
            } else if wParam == (HOTPLUG_DEVICE_TIMER_ID as _) {
                KillTimer((*pSysTray).GetHWnd(), (HOTPLUG_DEVICE_TIMER_ID as _));
                HotplugDeviceTimer(pSysTray);
            }
        }

        v if v == ID_ICON_HOTPLUG => {
            match lParam {
                l if (l) as u32 == WM_LBUTTONDOWN => {
                    SetTimer((*pSysTray).GetHWnd(), (HOTPLUG_TIMER_ID as _), GetDoubleClickTime(), None);
                }

                l if (l) as u32 == WM_LBUTTONUP => {
                }

                l if (l) as u32 == WM_LBUTTONDBLCLK => {
                    KillTimer((*pSysTray).GetHWnd(), (HOTPLUG_TIMER_ID as _));
                    _RunHotplug(pSysTray);
                }

                l if (l) as u32 == WM_RBUTTONDOWN => {
                }

                l if (l) as u32 == WM_RBUTTONUP => {
                    _ShowContextMenuR(pSysTray);
                }

                l if (l) as u32 == WM_RBUTTONDBLCLK => {
                }

                l if (l) as u32 == WM_MOUSEMOVE => {
                }

                _ => {}
            }
            return S_OK;
        }

        v if v == WM_DEVICECHANGE => {
            match wParam {
                w if w == (DBT_DEVNODES_CHANGED as WPARAM) => {
                    TRACE!("WM_DEVICECHANGE : DBT_DEVNODES_CHANGED\n");
                    SetTimer((*pSysTray).GetHWnd(), (HOTPLUG_DEVICE_TIMER_ID as _), 100, None);
                    *lResult = 1 as LRESULT;
                }

                w if w == (DBT_DEVICEARRIVAL as WPARAM) => {
                }
                w if w == (DBT_DEVICEQUERYREMOVE as WPARAM) => {
                }
                w if w == (DBT_DEVICEQUERYREMOVEFAILED as WPARAM) => {
                }
                w if w == (DBT_DEVICEREMOVECOMPLETE as WPARAM) => {
                    let mut strInfo = [0u16; 128];
                    _swprintf(strInfo.as_mut_ptr(), CPP_COM_WSTR_1DE543427BD7.as_ptr(), g_strMenuSel.as_mut_ptr());
                    NotifyBalloon(pSysTray, CPP_COM_WSTR_B87E4F458B30.as_ptr(), strInfo.as_ptr(), ID_ICON_HOTPLUG);

                    *lResult = 1 as LRESULT;
                }
                w if w == (DBT_DEVICEREMOVEPENDING as WPARAM) => {
                }
                _ => {}
            }
            return S_OK;
        }

        _ => {
            TRACE!("Hotplug_Message received for unknown ID %d, ignoring.\n");
            return S_FALSE;
        }
    }

    return S_FALSE;
}
