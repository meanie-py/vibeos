use crate::hotplug::eject::GetDeviceInstForRemoval;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut GUID_DEVCLASS_UNKNOWN: GUID = _GUID {
    Data1: 0x4d36e97e as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xe325 as ::core::ffi::c_ushort,
    Data3: 0x11ce as ::core::ffi::c_ushort,
    Data4: [
        0xbf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub const MAX_DEVICE_DISPLAYNAME_LEN: usize = 256;

unsafe extern "C" fn GetDeviceDisplayInfo(
    mut DevInst: DEVINST,
    mut pHotplugData: PHOTPLUG_DATA,
    mut pszDesc: LPWSTR,
    mut cchDesc: ULONG,
    mut pImageIndex: PINT,
) {
    let mut szGuidString: [WCHAR; (MAX_GUID_STRING_LEN as usize)] = [0; (MAX_GUID_STRING_LEN as usize)];
    let mut ClassGuid: GUID = GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    let mut ulSize: ULONG = 0;
    let mut cr: CONFIGRET = 0;

    /* Get the device description */
    ulSize = (cchDesc as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as ULONG;
    cr = CM_Get_DevNode_Registry_PropertyW(
        DevInst,
        CM_DRP_FRIENDLYNAME as ULONG,
        ::core::ptr::null_mut::<ULONG>(),
        pszDesc as PVOID,
        &raw mut ulSize,
        0 as ULONG,
    );
    if cr != CR_SUCCESS as CONFIGRET {
        ulSize = (cchDesc as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as ULONG;
        cr = CM_Get_DevNode_Registry_PropertyW(
            DevInst,
            CM_DRP_DEVICEDESC as ULONG,
            ::core::ptr::null_mut::<ULONG>(),
            pszDesc as PVOID,
            &raw mut ulSize,
            0 as ULONG,
        );
        if cr != CR_SUCCESS as CONFIGRET {
            LoadStringW(
                hApplet,
                IDS_UNKNOWN_DEVICE as UINT,
                pszDesc,
                cchDesc as ::core::ffi::c_int,
            );
        }
    }

    /* Get the class GUID */
    ulSize = ::core::mem::size_of::<[WCHAR; (MAX_GUID_STRING_LEN as usize)]>() as ULONG;
    cr = CM_Get_DevNode_Registry_PropertyW(
        DevInst,
        CM_DRP_CLASSGUID as ULONG,
        ::core::ptr::null_mut::<ULONG>(),
        &raw mut szGuidString as *mut WCHAR as PVOID,
        &raw mut ulSize,
        0 as ULONG,
    );
    if cr == CR_SUCCESS as CONFIGRET {
        pSetupGuidFromString(
            &raw mut szGuidString as *mut WCHAR as PCWSTR,
            &raw mut ClassGuid,
        );
    } else {
        ClassGuid = GUID_DEVCLASS_UNKNOWN;
    }

    /* Get the image for the class this device is in */
    SetupDiGetClassImageIndex(
        &raw mut (*pHotplugData).ImageListData,
        &raw mut ClassGuid,
        pImageIndex,
    );
}

unsafe extern "C" fn InsertDeviceTreeItem(
    hParent: HTREEITEM,
    DevInst: DEVINST,
    pHotplugData: PHOTPLUG_DATA,
) -> HTREEITEM {
    let mut szDisplayName: [WCHAR; MAX_DEVICE_DISPLAYNAME_LEN] = [0; MAX_DEVICE_DISPLAYNAME_LEN];
    let mut nClassImage: INT = 0;
    let mut tvItem: TVINSERTSTRUCTW = core::mem::zeroed();

    GetDeviceDisplayInfo(
        DevInst,
        pHotplugData,
        core::ptr::addr_of_mut!(szDisplayName) as LPWSTR,
        ARRAYSIZE!(szDisplayName) as ULONG,
        core::ptr::addr_of_mut!(nClassImage) as PINT,
    );

    ZeroMemory!(
        core::ptr::addr_of_mut!(tvItem) as *mut core::ffi::c_void,
        core::mem::size_of::<TVINSERTSTRUCTW>(),
    );
    tvItem.hParent = hParent;
    tvItem.hInsertAfter = TVI_LAST;

    tvItem.__bindgen_anon_1.item.mask = TVIF_STATE | TVIF_TEXT | TVIF_PARAM | TVIF_IMAGE | TVIF_SELECTEDIMAGE;
    tvItem.__bindgen_anon_1.item.state = TVIS_EXPANDED;
    tvItem.__bindgen_anon_1.item.stateMask = TVIS_EXPANDED;
    tvItem.__bindgen_anon_1.item.pszText = core::ptr::addr_of_mut!(szDisplayName) as *mut WCHAR as LPWSTR;
    tvItem.__bindgen_anon_1.item.iImage = nClassImage;
    tvItem.__bindgen_anon_1.item.iSelectedImage = nClassImage;
    tvItem.__bindgen_anon_1.item.lParam = DevInst as LPARAM;

    return TreeView_InsertItem!((*pHotplugData).hwndDeviceTree, core::ptr::addr_of_mut!(tvItem));
}

unsafe extern "C" fn DevTreeRecursiveInsertSubDevices(
    hParentItem: HTREEITEM,
    ParentDevInst: DEVINST,
    pHotplugData: PHOTPLUG_DATA,
) {
    let mut hTreeItem: HTREEITEM = core::ptr::null_mut();
    let mut ChildDevInst: DEVINST = 0;
    let mut cr: CONFIGRET;

    cr = CM_Get_Child(core::ptr::addr_of_mut!(ChildDevInst), ParentDevInst, 0);
    if cr != CR_SUCCESS {
        return;
    }

    hTreeItem = InsertDeviceTreeItem(hParentItem, ChildDevInst, pHotplugData);
    if !hTreeItem.is_null() {
        DevTreeRecursiveInsertSubDevices(hTreeItem, ChildDevInst, pHotplugData);
    }

    loop {
        cr = CM_Get_Sibling(core::ptr::addr_of_mut!(ChildDevInst), ChildDevInst, 0);
        if cr != CR_SUCCESS {
            return;
        }

        hTreeItem = InsertDeviceTreeItem(hParentItem, ChildDevInst, pHotplugData);
        if !hTreeItem.is_null() {
            DevTreeRecursiveInsertSubDevices(hTreeItem, ChildDevInst, pHotplugData);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn EnumHotpluggedDevices(mut pHotplugData: PHOTPLUG_DATA) {
    let mut did: SP_DEVINFO_DATA = core::mem::zeroed();
    let mut hdev: HDEVINFO = core::ptr::null_mut();
    let mut idev: core::ffi::c_int = 0;
    let mut dwCapabilities: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut ulStatus: ULONG = 0;
    let mut ulProblem: ULONG = 0;
    let mut hTreeItem: HTREEITEM = core::ptr::null_mut();
    let mut cr: CONFIGRET = 0;

    TreeView_DeleteAllItems!((*pHotplugData).hwndDeviceTree);

    hdev = SetupDiGetClassDevs(
        core::ptr::null::<GUID>(),
        core::ptr::null::<WCHAR>(),
        0 as HWND,
        (DIGCF_ALLCLASSES | DIGCF_PRESENT) as DWORD,
    );
    if hdev == INVALID_HANDLE_VALUE as HDEVINFO {
        return;
    }

    did.cbSize = core::mem::size_of::<SP_DEVINFO_DATA>() as DWORD;

    /* Enumerate all the attached devices */
    idev = 0 as core::ffi::c_int;
    while SetupDiEnumDeviceInfo(hdev, idev as DWORD, &raw mut did) != 0 {
        ulStatus = 0 as ULONG;
        ulProblem = 0 as ULONG;

        cr = CM_Get_DevNode_Status(
            &raw mut ulStatus,
            &raw mut ulProblem,
            did.DevInst as DEVINST,
            0 as ULONG,
        );
        if !(cr != CR_SUCCESS as CONFIGRET) {
            dwCapabilities = 0 as DWORD;
            dwSize = core::mem::size_of::<DWORD>() as DWORD;
            cr = CM_Get_DevNode_Registry_Property(
                did.DevInst as DEVINST,
                CM_DRP_CAPABILITIES as ULONG,
                core::ptr::null_mut::<ULONG>(),
                &raw mut dwCapabilities as PVOID,
                &raw mut dwSize,
                0 as ULONG,
            );
            if !(cr != CR_SUCCESS as CONFIGRET) {
                /* Add devices that require safe removal to the device tree */
                if dwCapabilities & CM_DEVCAP_REMOVABLE as DWORD != 0
                    && dwCapabilities & CM_DEVCAP_DOCKDEVICE as DWORD == 0
                    && dwCapabilities & CM_DEVCAP_SURPRISEREMOVALOK as DWORD == 0
                    && (dwCapabilities & CM_DEVCAP_EJECTSUPPORTED as DWORD != 0
                        || ulStatus & DN_DISABLEABLE as ULONG != 0)
                    && ulProblem == 0 as ULONG
                {
                    hTreeItem = InsertDeviceTreeItem(
                        TVI_ROOT as HTREEITEM,
                        did.DevInst as DEVINST,
                        pHotplugData,
                    );

                    if !hTreeItem.is_null()
                        && (*pHotplugData).dwFlags & HOTPLUG_DISPLAY_DEVICE_COMPONENTS as DWORD != 0
                    {
                        DevTreeRecursiveInsertSubDevices(
                            hTreeItem,
                            did.DevInst as DEVINST,
                            pHotplugData,
                        );
                    }
                }
            }
        }
        idev += 1;
    }

    SetupDiDestroyDeviceInfoList(hdev);
}

unsafe extern "C" fn InsertConfirmDeviceListItem(
    hwndCfmDeviceList: HWND,
    DevInst: DEVINST,
    pHotplugData: PHOTPLUG_DATA,
) {
    let mut szDisplayName: [WCHAR; MAX_DEVICE_DISPLAYNAME_LEN] = [0; MAX_DEVICE_DISPLAYNAME_LEN];
    let mut nClassImage: INT = 0;
    let mut lvItem: LVITEMW = core::mem::zeroed();

    GetDeviceDisplayInfo(
        DevInst,
        pHotplugData,
        szDisplayName.as_mut_ptr() as LPWSTR,
        ARRAYSIZE!(szDisplayName) as ULONG,
        core::ptr::addr_of_mut!(nClassImage),
    );

    ZeroMemory!(
        core::ptr::addr_of_mut!(lvItem) as *mut ::core::ffi::c_void,
        core::mem::size_of::<LVITEMW>(),
    );
    lvItem.mask = (LVIF_TEXT | LVIF_IMAGE | LVIF_PARAM) as UINT;
    lvItem.iItem = ListView_GetItemCount!(hwndCfmDeviceList) as INT;
    lvItem.pszText = szDisplayName.as_mut_ptr() as LPWSTR;
    lvItem.iImage = nClassImage;
    lvItem.lParam = DevInst as LPARAM;

    ListView_InsertItem!(hwndCfmDeviceList, core::ptr::addr_of_mut!(lvItem));
}

unsafe extern "C" fn CfmListRecursiveInsertSubDevices(
    mut hwndCfmDeviceList: HWND,
    mut ParentDevInst: DEVINST,
    mut pHotplugData: PHOTPLUG_DATA,
) {
    let mut ChildDevInst: DEVINST = 0;
    let mut cr: CONFIGRET;

    cr = CM_Get_Child(core::ptr::addr_of_mut!(ChildDevInst), ParentDevInst, 0);
    if cr != CR_SUCCESS {
        return;
    }

    InsertConfirmDeviceListItem(hwndCfmDeviceList, ChildDevInst, pHotplugData);
    CfmListRecursiveInsertSubDevices(hwndCfmDeviceList, ChildDevInst, pHotplugData);

    loop {
        cr = CM_Get_Sibling(core::ptr::addr_of_mut!(ChildDevInst), ChildDevInst, 0);
        if cr != CR_SUCCESS {
            return;
        }

        InsertConfirmDeviceListItem(hwndCfmDeviceList, ChildDevInst, pHotplugData);
        CfmListRecursiveInsertSubDevices(hwndCfmDeviceList, ChildDevInst, pHotplugData);
    }
}
#[no_mangle]
pub unsafe extern "C" fn CfmListEnumDevices(
    hwndCfmDeviceList: HWND,
    pHotplugData: PHOTPLUG_DATA,
) {
    let mut DevInst: DEVINST;

    DevInst = GetDeviceInstForRemoval(pHotplugData);
    if DevInst != 0 {
        InsertConfirmDeviceListItem(hwndCfmDeviceList, DevInst, pHotplugData);
        CfmListRecursiveInsertSubDevices(hwndCfmDeviceList, DevInst, pHotplugData);
    }
}
