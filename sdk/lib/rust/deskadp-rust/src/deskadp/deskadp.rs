use crate::deskadp::shxiface::IDeskDisplayAdapter_InitIface;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

// #define NDEBUG (flag-only macro with no value; no Rust declaration emitted)

pub type PDEVICEPROPERTIESW = unsafe extern "system" fn(HWND, LPCWSTR, LPCWSTR, BOOL) -> INT_PTR;

unsafe extern "C" fn GetColorDescription(
    lpDevMode: PDEVMODEW,
    lpBuffer: LPTSTR,
    dwBufferLen: DWORD,
) {
    let mut uid: UINT = 0 as UINT;

    if (*lpDevMode).dmBitsPerPel as u32 == 4 as u32 {
        uid = IDS_4BPP as UINT;
    } else if (*lpDevMode).dmBitsPerPel as u32 == 8 as u32 {
        uid = IDS_8BPP as UINT;
    } else if (*lpDevMode).dmBitsPerPel as u32 == 15 as u32 {
        uid = IDS_15BPP as UINT;
    } else if (*lpDevMode).dmBitsPerPel as u32 == 16 as u32 {
        uid = IDS_16BPP as UINT;
    } else if (*lpDevMode).dmBitsPerPel as u32 == 24 as u32 {
        uid = IDS_24BPP as UINT;
    } else if (*lpDevMode).dmBitsPerPel as u32 == 32 as u32 {
        uid = IDS_32BPP as UINT;
    }

    if uid == 0 as UINT
        || LoadString(
            hInstance,
            uid,
            lpBuffer as LPWSTR,
            dwBufferLen as ::core::ffi::c_int,
        ) == 0
    {
        *lpBuffer.offset(0 as isize) = ('\0' as u16);
    }
}

unsafe extern "C" fn GetRefreshRateDescription(
    mut lpDevMode: PDEVMODEW,
    mut lpBuffer: LPTSTR,
    mut dwBufferLen: DWORD,
) {
    let mut szFormat: [TCHAR; 64] = [0; 64];

    if (*lpDevMode).dmDisplayFrequency <= 1 {
        if LoadString(
            hInstance,
            IDS_DEFREFRESHRATE as UINT,
            lpBuffer,
            dwBufferLen as ::core::ffi::c_int,
        ) == 0
        {
            *lpBuffer.offset(0 as isize) = ('\0' as u16);
        }
    } else {
        if LoadString(
            hInstance,
            IDES_REFRESHRATEFMT as UINT,
            addr_of_mut!(szFormat) as LPTSTR,
            (core::mem::size_of::<[TCHAR; 64]>()
                / core::mem::size_of::<TCHAR>()) as ::core::ffi::c_int,
        ) == 0
        {
            szFormat[0 as usize] = ('\0' as u16);
        }

        _sntprintf(
            lpBuffer,
            (dwBufferLen) as _,
            addr_of_mut!(szFormat) as *mut TCHAR,
            (*lpDevMode).dmDisplayFrequency,
        );
    }
}

unsafe extern "C" fn InitListAllModesDialog(
    mut This: PDESKDISPLAYADAPTER,
    mut hwndListAllModesDlg: HWND,
) {
    let mut szFormat: [TCHAR; 64] = [0 as TCHAR; 64];
    let mut szBuffer: [TCHAR; 64] = [0 as TCHAR; 64];
    let mut szColors: [TCHAR; 64] = [0 as TCHAR; 64];
    let mut szRefreshRate: [TCHAR; 64] = [0 as TCHAR; 64];
    let mut lpDevMode: PDEVMODEW = core::ptr::null_mut();
    let mut lpCurrentDevMode: PDEVMODEW = core::ptr::null_mut();
    let mut dwIndex: DWORD = 0 as DWORD;
    let mut i: INT = 0;

    if !(*This).DeskExtInterface.is_null() {
        if LoadString(
            hInstance,
            IDS_MODEFMT as UINT,
            szFormat.as_mut_ptr(),
            (core::mem::size_of::<[TCHAR; 64]>()
                / core::mem::size_of_val(&szFormat[0 as usize]))
                as ::core::ffi::c_int,
        ) == 0
        {
            szFormat[0 as usize] = ('\0' as u16);
        }

        lpCurrentDevMode = (*(*This).DeskExtInterface)
            .GetCurrentMode
            .unwrap_unchecked()(
            (*(*This).DeskExtInterface).Context
        );

        loop {
            let fresh_index = dwIndex;
            dwIndex = dwIndex.wrapping_add(1);
            lpDevMode = (*(*This).DeskExtInterface)
                .EnumAllModes
                .unwrap_unchecked()(
                (*(*This).DeskExtInterface).Context,
                fresh_index,
            );
            if !lpDevMode.is_null() {
                GetColorDescription(
                    lpDevMode,
                    szColors.as_mut_ptr(),
                    (core::mem::size_of::<[TCHAR; 64]>()
                        / core::mem::size_of_val(&szColors[0 as usize]))
                        as DWORD,
                );

                GetRefreshRateDescription(
                    lpDevMode,
                    szRefreshRate.as_mut_ptr(),
                    (core::mem::size_of::<[TCHAR; 64]>()
                        / core::mem::size_of_val(&szRefreshRate[0 as usize]))
                        as DWORD,
                );

                _sntprintf(
                    szBuffer.as_mut_ptr(),
                    core::mem::size_of::<[TCHAR; 64]>()
                        / core::mem::size_of_val(&szBuffer[0 as usize]),
                    szFormat.as_mut_ptr(),
                    (*lpDevMode).dmPelsWidth,
                    (*lpDevMode).dmPelsHeight,
                    szColors.as_mut_ptr(),
                    szRefreshRate.as_mut_ptr(),
                );

                i = SendDlgItemMessage(
                    hwndListAllModesDlg,
                    IDC_ALLVALIDMODES as INT,
                    LB_ADDSTRING as UINT,
                    0 as WPARAM,
                    szBuffer.as_mut_ptr() as LPARAM,
                ) as INT;

                if i >= 0 as INT {
                    SendDlgItemMessage(
                        hwndListAllModesDlg,
                        IDC_ALLVALIDMODES as INT,
                        LB_SETITEMDATA as UINT,
                        i as WPARAM,
                        lpDevMode as LPARAM,
                    );

                    if lpDevMode == lpCurrentDevMode {
                        SendDlgItemMessage(
                            hwndListAllModesDlg,
                            IDC_ALLVALIDMODES as INT,
                            LB_SETCURSEL as UINT,
                            i as WPARAM,
                            0 as LPARAM,
                        );
                    }
                }
            }

            if lpDevMode.is_null() {
                break;
            }
        }
    }
}

unsafe extern "C" fn ChangeSelectedMode(
    mut This: PDESKDISPLAYADAPTER,
    mut hwndListAllModesDlg: HWND,
) -> BOOL {
    let mut i: INT = 0;
    let mut lpSelDevMode: PDEVMODEW = core::ptr::null_mut();
    let mut bRet: BOOL = (FALSE as i32);
    i = SendDlgItemMessage(
        hwndListAllModesDlg,
        (IDC_ALLVALIDMODES as i32),
        LB_GETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as INT;
    if i >= 0 {
        lpSelDevMode = SendDlgItemMessage(
            hwndListAllModesDlg,
            (IDC_ALLVALIDMODES as i32),
            LB_GETITEMDATA as UINT,
            i as WPARAM,
            0 as LPARAM,
        ) as PDEVMODEW;
    }
    if !lpSelDevMode.is_null() {
        (*This).lpSelDevMode = lpSelDevMode;
        bRet = (TRUE as i32);
    }
    return bRet;
}

unsafe extern "system" fn ListAllModesDlgProc(
    mut hwndDlg: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    let mut This: PDESKDISPLAYADAPTER = core::mem::zeroed();
    let mut Ret: INT_PTR = 0 as INT_PTR;

    if uMsg != WM_INITDIALOG as UINT {
        This = GetWindowLongPtr(hwndDlg, DWLP_USER) as PDESKDISPLAYADAPTER;
    }

    match uMsg {
        WM_INITDIALOG => {
            This = lParam as PDESKDISPLAYADAPTER;
            SetWindowLongPtr(hwndDlg, DWLP_USER, This as LONG_PTR);

            InitListAllModesDialog(This, hwndDlg);
            Ret = TRUE as INT_PTR;
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDOK => {
                    if ChangeSelectedMode(This, hwndDlg) != 0 {
                        EndDialog(hwndDlg, IDOK as INT_PTR);
                    }
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, IDCANCEL as INT_PTR);
                }

                _ => {}
            }
        }

        WM_CLOSE => {
            EndDialog(hwndDlg, IDCANCEL as INT_PTR);
        }

        _ => {}
    }

    return Ret;
}

unsafe extern "C" fn ShowListAllModes(This: PDESKDISPLAYADAPTER) {
    let mut lpPrevSel: PDEVMODEW;
    lpPrevSel = (*This).lpSelDevMode;
    if !(*This).DeskExtInterface.is_null()
        && DialogBoxParam(
            hInstance,
            MAKEINTRESOURCE!(IDD_LISTALLMODES),
            (*This).hwndDlg,
            Some(
                ListAllModesDlgProc,
            ),
            This as LPARAM,
        ) == IDOK as INT_PTR
    {
        if lpPrevSel != (*This).lpSelDevMode {
            let _ = PropSheet_Changed!(
                GetParent((*This).hwndDlg),
                (*This).hwndDlg,
            );
            (*(*This).DeskExtInterface)
                .SetCurrentMode
                .unwrap_unchecked()(
                (*(*This).DeskExtInterface).Context,
                (*This).lpSelDevMode as *mut DEVMODEW,
            );
        }
    }
}

unsafe extern "C" fn ShowAdapterProperties(This: PDESKDISPLAYADAPTER) {
    let mut hDevMgr: HMODULE = ::core::ptr::null_mut::<HINSTANCE__>();
    let mut pDevicePropertiesW: Option<PDEVICEPROPERTIESW>;
    hDevMgr = LoadLibrary((['d' as u16, 'e' as u16, 'v' as u16, 'm' as u16, 'g' as u16, 'r' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0u16].as_ptr())) as HMODULE;
    if !hDevMgr.is_null() {
        pDevicePropertiesW = ::core::mem::transmute::<FARPROC, Option<PDEVICEPROPERTIESW>>(GetProcAddress(
            hDevMgr as HINSTANCE,
            b"DevicePropertiesW\0".as_ptr() as LPCSTR,
        ));
        if let Some(pDevicePropertiesW) = pDevicePropertiesW {
            pDevicePropertiesW(
                (*This).hwndDlg,
                ::core::ptr::null::<WCHAR>(),
                (*This).lpDeviceId as LPCWSTR,
                (FALSE as i32),
            );
        }

        FreeLibrary(hDevMgr);
    }
}

unsafe extern "C" fn InitDisplayAdapterDialog(mut This: PDESKDISPLAYADAPTER) {
    let mut lpAdapterName: LPTSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut lpNA: [TCHAR; 64] = [0; 64];
    (*This).lpDeviceId = QueryDeskCplString(
        (*This).pdtobj,
        RegisterClipboardFormat(DESK_EXT_DISPLAYID),
    );
    EnableWindow(
        GetDlgItem((*This).hwndDlg, (IDC_ADAPTERPROPERTIES as i32)),
        ((!(*This).lpDeviceId.is_null()
            && *(*This).lpDeviceId.offset(0 as isize) != ('\0' as u16)) as i32),
    );
    lpAdapterName = QueryDeskCplString(
        (*This).pdtobj,
        RegisterClipboardFormat(DESK_EXT_DISPLAYNAME),
    );
    if !lpAdapterName.is_null() {
        SetDlgItemText(
            (*This).hwndDlg,
            (IDC_ADAPTERNAME as i32),
            lpAdapterName,
        );

        LocalFree(lpAdapterName as HLOCAL);
    }

    LoadString(
        hInstance,
        IDS_NOTAVAIL,
        &raw mut lpNA as LPTSTR,
        _countof!(lpNA) as core::ffi::c_int,
    );

    if !(*This).DeskExtInterface.is_null() {
        SetDlgItemTextW(
            (*This).hwndDlg,
            (IDC_CHIPTYPE as i32),
            (if *(&raw mut (*(*This).DeskExtInterface).ChipType as *mut TCHAR) != 0 {
                &raw mut (*(*This).DeskExtInterface).ChipType as *mut TCHAR
            } else {
                &raw mut lpNA as *mut TCHAR
            }),
        );
        SetDlgItemTextW(
            (*This).hwndDlg,
            (IDC_DACTYPE as i32),
            (if *(&raw mut (*(*This).DeskExtInterface).DacType as *mut TCHAR) != 0 {
                &raw mut (*(*This).DeskExtInterface).DacType as *mut TCHAR
            } else {
                &raw mut lpNA as *mut TCHAR
            }),
        );
        SetDlgItemTextW(
            (*This).hwndDlg,
            (IDC_MEMORYSIZE as i32),
            (if *(&raw mut (*(*This).DeskExtInterface).MemorySize as *mut TCHAR) != 0 {
                &raw mut (*(*This).DeskExtInterface).MemorySize as *mut TCHAR
            } else {
                &raw mut lpNA as *mut TCHAR
            }),
        );
        SetDlgItemTextW(
            (*This).hwndDlg,
            (IDC_ADAPTERSTRING as i32),
            (if *(&raw mut (*(*This).DeskExtInterface).AdapterString as *mut TCHAR) != 0 {
                &raw mut (*(*This).DeskExtInterface).AdapterString as *mut TCHAR
            } else {
                &raw mut lpNA as *mut TCHAR
            }),
        );
        SetDlgItemTextW(
            (*This).hwndDlg,
            (IDC_BIOSINFORMATION as i32),
            (if *(&raw mut (*(*This).DeskExtInterface).BiosString as *mut TCHAR) != 0 {
                &raw mut (*(*This).DeskExtInterface).BiosString as *mut TCHAR
            } else {
                &raw mut lpNA as *mut TCHAR
            }),
        );

        (*This).lpDevModeOnInit = (*(*This).DeskExtInterface)
            .GetCurrentMode
            .unwrap_unchecked()(
            (*(*This).DeskExtInterface).Context
        );
    } else {
        (*This).lpDevModeOnInit = ::core::ptr::null_mut::<_devicemodeW>();
        SetDlgItemTextW((*This).hwndDlg, (IDC_CHIPTYPE as i32), &raw mut lpNA as *mut TCHAR);
        SetDlgItemTextW((*This).hwndDlg, (IDC_DACTYPE as i32), &raw mut lpNA as *mut TCHAR);
        SetDlgItemTextW((*This).hwndDlg, (IDC_MEMORYSIZE as i32), &raw mut lpNA as *mut TCHAR);
        SetDlgItemTextW((*This).hwndDlg, (IDC_ADAPTERSTRING as i32), &raw mut lpNA as *mut TCHAR);
        SetDlgItemTextW((*This).hwndDlg, (IDC_BIOSINFORMATION as i32), &raw mut lpNA as *mut TCHAR);
    }

    (*This).lpSelDevMode = (*This).lpDevModeOnInit;
}

unsafe extern "C" fn ApplyDisplayAdapterChanges(mut This: PDESKDISPLAYADAPTER) -> LONG {
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

    return PSNRET_INVALID_NOCHANGEPAGE as LONG;
}

unsafe extern "C" fn ResetDisplayAdapterChanges(This: PDESKDISPLAYADAPTER) {
    if !(*This).DeskExtInterface.is_null() && !(*This).lpDevModeOnInit.is_null() {
        (((*(*This).DeskExtInterface).SetCurrentMode).unwrap_unchecked())(
            (*(*This).DeskExtInterface).Context,
            (*This).lpDevModeOnInit,
        );
    }
}

unsafe extern "system" fn DisplayAdapterDlgProc(
    mut hwndDlg: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    let mut This: PDESKDISPLAYADAPTER = core::ptr::null_mut();
    let mut Ret: INT_PTR = 0 as INT_PTR;

    if uMsg != WM_INITDIALOG as UINT {
        This = GetWindowLongPtr(hwndDlg, DWLP_USER as ::core::ffi::c_int) as PDESKDISPLAYADAPTER;
    }

    match uMsg {
        WM_INITDIALOG => {
            This = (*(lParam as LPCPROPSHEETPAGE)).lParam as PDESKDISPLAYADAPTER;
            (*This).hwndDlg = hwndDlg;
            SetWindowLongPtr(hwndDlg, DWLP_USER as ::core::ffi::c_int, This as LONG_PTR);

            InitDisplayAdapterDialog(This);
            Ret = TRUE as INT_PTR;
        }

        WM_COMMAND => match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
            IDC_ADAPTERPROPERTIES => {
                ShowAdapterProperties(This);
            }
            IDC_LISTALLMODES => {
                ShowListAllModes(This);
            }
            _ => {}
        },

        WM_NOTIFY => {
            let mut nmh: *mut NMHDR = lParam as *mut NMHDR;

            match ((*nmh).code) as i32 {
                PSN_APPLY => {
                    SetWindowLongPtr(
                        hwndDlg,
                        DWLP_MSGRESULT as ::core::ffi::c_int,
                        ApplyDisplayAdapterChanges(This) as LONG_PTR,
                    );
                    Ret = TRUE as INT_PTR;
                }
                PSN_RESET => {
                    ResetDisplayAdapterChanges(This);
                }
                _ => {}
            }
        }

        _ => {}
    }

    return Ret;
}

unsafe extern "C" fn IDeskDisplayAdapter_Destroy(This: PDESKDISPLAYADAPTER) {
    if !(*This).pdtobj.is_null() {
        IDataObject_Release((*This).pdtobj);
        (*This).pdtobj = core::ptr::null_mut();
    }

    if !(*This).DeskExtInterface.is_null() {
        LocalFree((*This).DeskExtInterface as HLOCAL);
        (*This).DeskExtInterface = core::ptr::null_mut();
    }

    if !(*This).lpDeviceId.is_null() {
        LocalFree((*This).lpDeviceId as HLOCAL);
        (*This).lpDeviceId = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "C" fn IDeskDisplayAdapter_AddRef(This: PDESKDISPLAYADAPTER) -> ULONG {
    let mut ret: ULONG;

    ret = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as PLONG) as ULONG;
    if ret == 1 {
        InterlockedIncrement(core::ptr::addr_of_mut!(dll_refs) as PLONG);
    }

    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn IDeskDisplayAdapter_Release(This: PDESKDISPLAYADAPTER) -> ULONG {
    let mut ret: ULONG;

    ret = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as PLONG) as ULONG;
    if ret == 0 {
        IDeskDisplayAdapter_Destroy(This);
        InterlockedDecrement(core::ptr::addr_of_mut!(dll_refs) as PLONG);

        HeapFree(GetProcessHeap(),
                 0,
                 This as PVOID);
    }

    return ret;
}
#[no_mangle]
pub unsafe extern "system" fn IDeskDisplayAdapter_QueryInterface(This: PDESKDISPLAYADAPTER, iid: *const IID, pvObject: *mut PVOID) -> HRESULT {
    *pvObject = core::ptr::null_mut();

    if IsEqualIID!(iid, core::ptr::addr_of!(IID_IShellPropSheetExt)) != 0
        || IsEqualIID!(iid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *pvObject = impl_to_interface!(This, IShellPropSheetExt) as PVOID;
    }
    else if IsEqualIID!(iid, core::ptr::addr_of!(IID_IShellExtInit)) != 0
    {
        *pvObject = impl_to_interface!(This, IShellExtInit) as PVOID;
    }
    else if IsEqualIID!(iid, core::ptr::addr_of!(IID_IClassFactory)) != 0
    {
        *pvObject = impl_to_interface!(This, IClassFactory) as PVOID;
    }
    else
    {
        DPRINT1!("IDeskDisplayAdapter::QueryInterface(%p,%p): E_NOINTERFACE\n", iid, pvObject);
        return E_NOINTERFACE;
    }

    IDeskDisplayAdapter_AddRef(This);
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn IDeskDisplayAdapter_Initialize(mut This: PDESKDISPLAYADAPTER, mut pidlFolder: LPCITEMIDLIST, mut pdtobj: *mut IDataObject, mut hkeyProgID: HKEY) -> HRESULT {
    DPRINT1!("IDeskDisplayAdapter::Initialize(%p,%p,%p)\n", pidlFolder, pdtobj, hkeyProgID);

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
pub unsafe extern "C" fn IDeskDisplayAdapter_AddPages(This: PDESKDISPLAYADAPTER, pfnAddPage: LPFNADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
    let mut hpsp: HPROPSHEETPAGE = core::ptr::null_mut();
    let mut psp: PROPSHEETPAGE = core::mem::zeroed();

    DPRINT1!("IDeskDisplayAdapter::AddPages(%p,%p)\n", pfnAddPage, lParam);

    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    psp.dwFlags = PSP_DEFAULT;
    psp.hInstance = hInstance;
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_DISPLAYADAPTER);
    psp.pfnDlgProc = Some(DisplayAdapterDlgProc);
    psp.lParam = This as LPARAM;

    hpsp = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp) as LPCPROPSHEETPAGE);
    if !hpsp.is_null() && (pfnAddPage.unwrap_unchecked())(hpsp, lParam) != 0 {
        return S_OK;
    }

    return S_FALSE;
}

#[no_mangle]
pub unsafe extern "C" fn IDeskDisplayAdapter_ReplacePage(This: PDESKDISPLAYADAPTER, uPageID: EXPPS, pfnReplacePage: LPFNADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
    DPRINT1!("IDeskDisplayAdapter::ReplacePage(%u,%p,%p)\n", uPageID, pfnReplacePage, lParam);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IDeskDisplayAdapter_Constructor(riid: *const IID, mut ppv: *mut LPVOID) -> HRESULT {
    let mut This: PDESKDISPLAYADAPTER = ::core::ptr::null_mut::<_DESKDISPLAYADAPTER>();
    let mut hRet: HRESULT = E_OUTOFMEMORY;

    DPRINT1!("IDeskDisplayAdapter::Constructor(%p,%p)\n", riid, ppv);

    This = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        ::core::mem::size_of::<_DESKDISPLAYADAPTER>() as SIZE_T,
    ) as PDESKDISPLAYADAPTER;
    if !This.is_null() {
        ZeroMemory!(
            This as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<_DESKDISPLAYADAPTER>() as SIZE_T,
        );

        IDeskDisplayAdapter_InitIface(This);

        hRet = IDeskDisplayAdapter_QueryInterface(This, riid, ppv as *mut PVOID);
        if !(hRet >= 0) {
            IDeskDisplayAdapter_Release(This);
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
