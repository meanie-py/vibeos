// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn GetDeviceInstForRemoval(pHotplugData: PHOTPLUG_DATA) -> DEVINST {
    let mut hItem: HTREEITEM;
    let mut hParentItem: HTREEITEM;
    let mut tvItem: TVITEMW = core::mem::zeroed();

    hItem = TreeView_GetSelection!((*pHotplugData).hwndDeviceTree);
    if hItem.is_null() {
        return 0;
    }

    /* Find root item */
    hParentItem = TreeView_GetParent!((*pHotplugData).hwndDeviceTree, hItem);
    while !hParentItem.is_null() {
        hItem = hParentItem;
        hParentItem = TreeView_GetParent!((*pHotplugData).hwndDeviceTree, hItem);
    }

    ZeroMemory!(
        core::ptr::addr_of_mut!(tvItem) as *mut core::ffi::c_void,
        core::mem::size_of::<TVITEMW>(),
    );
    tvItem.mask = TVIF_PARAM;
    tvItem.hItem = hItem;

    TreeView_GetItem!((*pHotplugData).hwndDeviceTree, core::ptr::addr_of_mut!(tvItem));

    return tvItem.lParam as DEVINST;
}

unsafe extern "C" fn FillConfirmDeviceList(
    hwndCfmDeviceList: HWND,
    pHotplugData: PHOTPLUG_DATA,
) {
    let mut lvColumn: LVCOLUMNW = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(lvColumn) as *mut core::ffi::c_void,
        core::mem::size_of::<LVCOLUMNW>(),
    );
    lvColumn.mask = LVCF_FMT;
    lvColumn.fmt = (LVCFMT_LEFT | LVCFMT_IMAGE) as i32;
    ListView_InsertColumn!(
        hwndCfmDeviceList,
        0,
        core::ptr::addr_of_mut!(lvColumn),
    );

    CfmListEnumDevices(hwndCfmDeviceList, pHotplugData);

    ListView_SetColumnWidth!(
        hwndCfmDeviceList,
        0,
        LVSCW_AUTOSIZE_USEHEADER,
    );
}

unsafe extern "C" fn SafeRemoveDevice(DevInst: DEVINST, hwndDlg: HWND) {
    let mut VetoType: PNP_VETO_TYPE = PNP_VetoTypeUnknown;
    let mut cr: CONFIGRET;

    cr = CM_Request_Device_EjectW(
        DevInst,
        core::ptr::addr_of_mut!(VetoType),
        core::ptr::null_mut(),
        0,
        0,
    );
    if cr != CR_SUCCESS && VetoType == PNP_VetoTypeUnknown {
        let mut pszFormat: LPCWSTR = RUSTCONV_WSTR_96A296D224F2.as_ptr();
        let mut szError: [WCHAR; 64] = [0; 64];

        /* NOTES: IDS_EJECT_ERROR_FORMAT resource has to be explicitly NULL-terminated
         * so we can use the string directly without having to make a copy of it. */
        LoadStringW(
            hApplet,
            IDS_EJECT_ERROR_FORMAT,
            core::ptr::addr_of_mut!(pszFormat) as LPWSTR,
            0,
        );
        StringCbPrintfW!(
            core::ptr::addr_of_mut!(szError) as *mut WCHAR,
            core::mem::size_of::<[WCHAR; 64]>(),
            pszFormat,
            cr,
        );

        MessageBoxW(
            hwndDlg,
            core::ptr::addr_of_mut!(szError) as *mut WCHAR as LPCWSTR,
            core::ptr::null(),
            MB_ICONEXCLAMATION | MB_OK,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn ConfirmRemovalDlgProc(mut hwndDlg: HWND, mut uMsg: UINT, mut wParam: WPARAM, mut lParam: LPARAM) -> INT_PTR {
    let mut pHotplugData: PHOTPLUG_DATA = core::ptr::null_mut::<_HOTPLUG_DATA>();

    pHotplugData = GetWindowLongPtrW(hwndDlg, DWLP_USER) as PHOTPLUG_DATA;

    match uMsg {
        WM_INITDIALOG => {
            let mut hwndDevList: HWND = core::ptr::null_mut::<HWND__>();

            pHotplugData = lParam as PHOTPLUG_DATA;
            SetWindowLongPtrW(hwndDlg, DWLP_USER, pHotplugData as LONG_PTR);

            hwndDevList = GetDlgItem(hwndDlg, (IDC_CONFIRM_STOP_DEVICE_LIST as i32));

            ListView_SetImageList!(
                hwndDevList,
                (*pHotplugData).ImageListData.ImageList,
                LVSIL_SMALL,
            );

            FillConfirmDeviceList(hwndDevList, pHotplugData);

            return TRUE as INT_PTR;
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDOK => {
                    SafeRemoveDevice(GetDeviceInstForRemoval(pHotplugData), hwndDlg);
                    EndDialog(hwndDlg, LOWORD!(wParam) as INT_PTR);
                }
                IDCANCEL => {
                    EndDialog(hwndDlg, LOWORD!(wParam) as INT_PTR);
                }
                _ => {}
            }

            // break;
        }

        WM_DESTROY => {
            SetWindowLongPtrW(hwndDlg, DWLP_USER, 0 as LONG_PTR);
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
