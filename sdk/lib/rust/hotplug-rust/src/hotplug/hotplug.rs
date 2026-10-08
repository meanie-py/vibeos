// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_960B7A428431: [u16; 11] = [
    0x0064u16, 0x0065, 0x0076, 0x006D, 0x0067, 0x0072, 0x002E, 0x0064, 0x006C, 0x006C,
    0x0000,
];
pub static RUSTCONV_WSTR_9903323D415F: [u16; 13] = [
    0x0048u16, 0x006F, 0x0074, 0x0050, 0x006C, 0x0075, 0x0067, 0x0046, 0x006C, 0x0061,
    0x0067, 0x0073, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hApplet: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut Applets: [APPLET; 1] = unsafe {
    [APPLET {
        idIcon: (IDI_HOTPLUG as i32),
        idName: (IDS_CPLNAME as i32),
        idDescription: (IDS_CPLDESCRIPTION as i32),
        AppletProc: Some(InitApplet),
    }]
};

// #define NDEBUG (flag-only macro with no value, no Rust const emitted)


unsafe extern "C" fn GetHotPlugFlags() -> DWORD {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwFlags: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut dwError: DWORD = 0;
    'done: {
        dwError = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            REGSTR_PATH_SYSTRAY,
            0,
            KEY_READ,
            core::ptr::addr_of_mut!(hKey),
        ) as DWORD;
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        dwError = RegQueryValueExW(
            hKey,
            RUSTCONV_WSTR_9903323D415F.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwFlags) as LPBYTE,
            core::ptr::addr_of_mut!(dwSize),
        ) as DWORD;
        if dwError != ERROR_SUCCESS {
            dwFlags = 0;
        }
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    return dwFlags;
}

unsafe extern "C" fn SetHotPlugFlags(mut dwFlags: DWORD) -> DWORD {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwError: DWORD;

    'done: {
        dwError = (RegCreateKeyExW(
            HKEY_CURRENT_USER,
            REGSTR_PATH_SYSTRAY,
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            core::ptr::null_mut(),
            &raw mut hKey,
            core::ptr::null_mut(),
        )) as u32;
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        dwError = (RegSetValueExW(
            hKey,
            RUSTCONV_WSTR_9903323D415F.as_ptr(),
            0,
            REG_DWORD,
            &raw mut dwFlags as LPBYTE,
            core::mem::size_of::<DWORD>() as DWORD,
        )) as u32;
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    return dwError;
}

unsafe extern "C" fn UpdateDialog(hwndDlg: HWND) {
    let mut hwndDeviceTree: HWND;
    let mut bHasItem: BOOL;

    hwndDeviceTree = GetDlgItem(hwndDlg, (IDC_SAFE_REMOVE_DEVICE_TREE as i32));

    bHasItem = (TreeView_GetCount!(hwndDeviceTree) != 0) as BOOL;
    if bHasItem != 0 {
        TreeView_SelectItem!(hwndDeviceTree, TreeView_GetFirstVisible!(hwndDeviceTree));
    }

    EnableWindow(GetDlgItem(hwndDlg, (IDC_SAFE_REMOVE_PROPERTIES as i32)), bHasItem);
    EnableWindow(GetDlgItem(hwndDlg, (IDC_SAFE_REMOVE_STOP as i32)), bHasItem);
}

unsafe extern "C" fn ShowContextMenu(
    hwndDlg: HWND,
    hwndTreeView: HWND,
    pHotplugData: PHOTPLUG_DATA,
) {
    let mut hTreeItem: HTREEITEM;
    let mut rc: RECT = core::mem::zeroed();
    let mut pt: POINT = core::mem::zeroed();

    hTreeItem = TreeView_GetSelection!(hwndTreeView);
    if hTreeItem.is_null() {
        return;
    }

    TreeView_GetItemRect!(hwndTreeView, hTreeItem, core::ptr::addr_of_mut!(rc), TRUE);

    pt.x = (rc.left + rc.right) / 2;
    pt.y = (rc.top + rc.bottom) / 2;
    ClientToScreen(hwndTreeView, core::ptr::addr_of_mut!(pt));
    TrackPopupMenu(
        GetSubMenu((*pHotplugData).hPopupMenu, 0),
        TPM_LEFTALIGN | TPM_TOPALIGN,
        pt.x,
        pt.y,
        0,
        hwndDlg,
        core::ptr::null(),
    );
}

unsafe extern "C" fn GetSelectedDeviceInst(pHotplugData: PHOTPLUG_DATA) -> DEVINST {
    let mut hTreeItem: HTREEITEM;
    let mut item: TVITEMW = core::mem::zeroed();

    hTreeItem = TreeView_GetSelection!((*pHotplugData).hwndDeviceTree);
    if hTreeItem.is_null() {
        return 0;
    }

    ZeroMemory!(
        core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void,
        core::mem::size_of::<TVITEMW>() as usize,
    );
    item.mask = TVIF_PARAM;
    item.hItem = hTreeItem;

    TreeView_GetItem!((*pHotplugData).hwndDeviceTree, core::ptr::addr_of_mut!(item));

    return item.lParam as DEVINST;
}

unsafe extern "C" fn ShowDeviceProperties(hwndParent: HWND, DevInst: DEVINST) {
    type PFDEVICEPROPERTIESW = unsafe extern "system" fn(HWND, LPCWSTR, LPCWSTR, BOOL) -> i32;
    let mut ulSize: ULONG = 0;
    let mut cr: CONFIGRET = 0;
    let mut pszDevId: LPWSTR = core::ptr::null_mut::<WCHAR>();
    cr = CM_Get_Device_ID_Size(core::ptr::addr_of_mut!(ulSize), DevInst, 0);
    if cr != CR_SUCCESS || ulSize == 0 {
        return;
    }
    /* Take the terminating NULL into account */
    ulSize = ulSize.wrapping_add(1);
    pszDevId = HeapAlloc(
        GetProcessHeap(),
        0,
        (ulSize as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if pszDevId.is_null() {
        return;
    }
    cr = CM_Get_Device_IDW(DevInst, pszDevId, ulSize, 0);
    if cr == CR_SUCCESS {
        let mut hDevMgrDll: HMODULE = core::ptr::null_mut::<HINSTANCE__>();
        let mut pDevicePropertiesW: Option<PFDEVICEPROPERTIESW> = None;

        hDevMgrDll = LoadLibraryW(RUSTCONV_WSTR_960B7A428431.as_ptr()) as HMODULE;
        if !hDevMgrDll.is_null() {
            pDevicePropertiesW = core::mem::transmute::<FARPROC, Option<PFDEVICEPROPERTIESW>>(
                GetProcAddress(
                    hDevMgrDll as HINSTANCE,
                    b"DevicePropertiesW\0".as_ptr() as LPCSTR,
                ),
            );
            if let Some(pDevicePropertiesW) = pDevicePropertiesW {
                pDevicePropertiesW(
                    hwndParent,
                    core::ptr::null_mut(),
                    pszDevId,
                    (FALSE) as i32,
                );
            }

            FreeLibrary(hDevMgrDll);
        }
    }

    HeapFree(GetProcessHeap(), 0, pszDevId as PVOID);
}
#[no_mangle]
pub unsafe extern "system" fn SafeRemovalDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pHotplugData: PHOTPLUG_DATA = core::ptr::null_mut();

    pHotplugData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PHOTPLUG_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pHotplugData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                core::mem::size_of::<HOTPLUG_DATA>() as SIZE_T,
            ) as PHOTPLUG_DATA;
            if !pHotplugData.is_null() {
                let mut szWindowTitle: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

                SetWindowLongPtr(hwndDlg, DWLP_USER, pHotplugData as LONG_PTR);

                if LoadStringW(
                    hApplet,
                    IDS_CPLNAME,
                    core::ptr::addr_of_mut!(szWindowTitle) as LPWSTR,
                    (ARRAYSIZE!(szWindowTitle)) as INT,
                ) != 0
                {
                    SetWindowTextW(hwndDlg, core::ptr::addr_of_mut!(szWindowTitle) as LPCWSTR);
                }

                (*pHotplugData).hIcon = LoadImageW(
                    hApplet,
                    MAKEINTRESOURCEW!(IDI_HOTPLUG),
                    IMAGE_ICON,
                    GetSystemMetrics((SM_CXICON) as i32),
                    GetSystemMetrics((SM_CYICON) as i32),
                    LR_DEFAULTCOLOR,
                ) as HICON;
                (*pHotplugData).hIconSm = LoadImageW(
                    hApplet,
                    MAKEINTRESOURCEW!(IDI_HOTPLUG),
                    IMAGE_ICON,
                    GetSystemMetrics((SM_CXSMICON) as i32),
                    GetSystemMetrics((SM_CYSMICON) as i32),
                    LR_DEFAULTCOLOR,
                ) as HICON;
                SendMessageW(
                    hwndDlg,
                    WM_SETICON,
                    (ICON_BIG) as WPARAM,
                    (*pHotplugData).hIcon as LPARAM,
                );
                SendMessageW(
                    hwndDlg,
                    WM_SETICON,
                    (ICON_SMALL) as WPARAM,
                    (*pHotplugData).hIconSm as LPARAM,
                );

                (*pHotplugData).ImageListData.cbSize =
                    core::mem::size_of::<SP_CLASSIMAGELIST_DATA>() as DWORD;
                SetupDiGetClassImageList(core::ptr::addr_of_mut!((*pHotplugData).ImageListData));

                (*pHotplugData).hPopupMenu = LoadMenu(
                    hApplet,
                    MAKEINTRESOURCE!(IDM_POPUP_DEVICE_TREE),
                );
                (*pHotplugData).hwndDeviceTree =
                    GetDlgItem(hwndDlg, (IDC_SAFE_REMOVE_DEVICE_TREE as i32));
                (*pHotplugData).dwFlags = GetHotPlugFlags();

                if ((*pHotplugData).dwFlags & HOTPLUG_DISPLAY_DEVICE_COMPONENTS) != 0 {
                    Button_SetCheck!(
                        GetDlgItem(hwndDlg, (IDC_SAFE_REMOVE_DISPLAY_COMPONENTS as i32)),
                        BST_CHECKED,
                    );
                }

                TreeView_SetImageList!(
                    (*pHotplugData).hwndDeviceTree,
                    (*pHotplugData).ImageListData.ImageList,
                    TVSIL_NORMAL,
                );

                EnumHotpluggedDevices(pHotplugData);
                UpdateDialog(hwndDlg);
            }
            return (TRUE) as INT_PTR;
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDCLOSE => {
                    KillTimer(hwndDlg, 1);
                    EndDialog(hwndDlg, (TRUE) as INT_PTR);
                }

                IDC_SAFE_REMOVE_DISPLAY_COMPONENTS => {
                    if (HIWORD!(wParam)) as u32 == BN_CLICKED {
                        if !pHotplugData.is_null() {
                            if (Button_GetCheck!(GetDlgItem(hwndDlg, (IDC_SAFE_REMOVE_DISPLAY_COMPONENTS as i32)))) as u32
                                == BST_CHECKED
                            {
                                (*pHotplugData).dwFlags |= HOTPLUG_DISPLAY_DEVICE_COMPONENTS;
                            } else {
                                (*pHotplugData).dwFlags &= !HOTPLUG_DISPLAY_DEVICE_COMPONENTS;
                            }

                            SetHotPlugFlags((*pHotplugData).dwFlags);

                            EnumHotpluggedDevices(pHotplugData);
                            UpdateDialog(hwndDlg);
                        }
                    }
                }

                IDC_SAFE_REMOVE_PROPERTIES | IDM_PROPERTIES => {
                    ShowDeviceProperties(hwndDlg, GetSelectedDeviceInst(pHotplugData));
                }

                IDC_SAFE_REMOVE_STOP | IDM_STOP => {
                    if !pHotplugData.is_null() {
                        DialogBoxParamW(
                            hApplet,
                            MAKEINTRESOURCEW!(IDD_CONFIRM_STOP_HARDWARE_DIALOG),
                            hwndDlg,
                            Some(
                                ConfirmRemovalDlgProc,
                            ),
                            pHotplugData as LPARAM,
                        );
                    }
                }

                _ => {}
            }
            // break;
        }

        WM_DEVICECHANGE => {
            match wParam as UINT {
                DBT_DEVNODES_CHANGED => {
                    SetTimer(
                        hwndDlg,
                        1,
                        500,
                        None,
                    );
                }
                _ => {}
            }
            // break;
        }

        WM_TIMER => {
            if wParam == 1 {
                KillTimer(hwndDlg, 1);

                if !pHotplugData.is_null() {
                    EnumHotpluggedDevices(pHotplugData);
                    UpdateDialog(hwndDlg);
                }
            }
            // break;
        }

        WM_NOTIFY => {
            if (*(lParam as LPNMHDR)).idFrom == (IDC_SAFE_REMOVE_DEVICE_TREE as UINT_PTR) {
                if (*(lParam as LPNMHDR)).code == NM_RCLICK {
                    if !pHotplugData.is_null() {
                        ShowContextMenu(
                            hwndDlg,
                            (*(lParam as LPNMHDR)).hwndFrom,
                            pHotplugData,
                        );
                        return (TRUE) as INT_PTR;
                    }
                }
            }
            // break;
        }

        WM_CLOSE => {
            KillTimer(hwndDlg, 1);
            EndDialog(hwndDlg, (TRUE) as INT_PTR);
            // break;
        }

        WM_DESTROY => {
            if !pHotplugData.is_null() {
                if !(*pHotplugData).hPopupMenu.is_null() {
                    DestroyMenu((*pHotplugData).hPopupMenu);
                }

                SetupDiDestroyClassImageList(core::ptr::addr_of_mut!((*pHotplugData).ImageListData));

                if !(*pHotplugData).hIconSm.is_null() {
                    DestroyIcon((*pHotplugData).hIconSm);
                }

                if !(*pHotplugData).hIcon.is_null() {
                    DestroyIcon((*pHotplugData).hIcon);
                }

                HeapFree(
                    GetProcessHeap(),
                    0,
                    pHotplugData as PVOID,
                );
                SetWindowLongPtr(hwndDlg, DWLP_USER, 0 as LONG_PTR);
            }
            // break;
        }

        _ => {}
    }

    return (FALSE) as INT_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn InitApplet(
    hwnd: HWND,
    uMsg: UINT,
    wParam: LPARAM,
    lParam: LPARAM,
) -> LONG {
    // DPRINT("InitApplet()\n"); -- compiled out under NDEBUG
    DialogBox!(
        hApplet,
        MAKEINTRESOURCE!(IDD_SAFE_REMOVE_HARDWARE_DIALOG),
        hwnd,
        Some(SafeRemovalDlgProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR),
    );

    // TODO
    return TRUE as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn CPlApplet(
    hwndCPl: HWND,
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
            if i < NUM_APPLETS as UINT {
                let CPlInfo: *mut CPLINFO = lParam2 as *mut CPLINFO;
                (*CPlInfo).lData = 0 as LONG_PTR;
                (*CPlInfo).idIcon = Applets[i as usize].idIcon;
                (*CPlInfo).idName = Applets[i as usize].idName;
                (*CPlInfo).idInfo = Applets[i as usize].idDescription;
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
pub unsafe extern "system" fn DllMain(
    mut hinstDLL: HINSTANCE,
    mut dwReason: DWORD,
    mut lpvReserved: LPVOID,
) -> INT {
    UNREFERENCED_PARAMETER!(lpvReserved);

    match dwReason {
        DLL_PROCESS_ATTACH | DLL_THREAD_ATTACH => {
            hApplet = hinstDLL;
        }
        _ => {}
    }
    return (TRUE) as INT;
}
