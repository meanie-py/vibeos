use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// #define NONAMELESSUNION (empty object-like macro, no Rust declaration emitted)

#[repr(C)]
pub struct DeviceData {
    pub nobjects: i32,
    pub lpdid: *mut IDirectInputDevice8W,
    pub ddi: DIDEVICEINSTANCEW,
    pub ddo: [DIDEVICEOBJECTINSTANCEW; 256],
    /// ActionFormat for every user.
    /// In same order as ConfigureDevicesData usernames
    pub user_afs: *mut DIACTIONFORMATW,
}

#[repr(C)]
pub struct DIDevicesData {
    pub ndevices: i32,
    pub devices: *mut DeviceData,
}

#[repr(C)]
pub struct ConfigureDevicesData {
    pub lpDI: *mut IDirectInput8W,
    pub original_lpdiaf: LPDIACTIONFORMATW,
    pub devices_data: DIDevicesData,
    pub display_only: i32,
    pub nusernames: i32,
    pub usernames: *mut *mut WCHAR,
}

unsafe extern "system" fn collect_objects(lpddo: LPCDIDEVICEOBJECTINSTANCEW, pvRef: LPVOID) -> BOOL {
    let data: *mut DeviceData = pvRef as *mut DeviceData;

    (*data).ddo[(*data).nobjects as usize] = *lpddo;

    (*data).nobjects += 1;
    return (DIENUM_CONTINUE) as BOOL;
}

unsafe extern "system" fn collect_devices(
    lpddi: LPCDIDEVICEINSTANCEW,
    lpdid: *mut IDirectInputDevice8W,
    dwFlags: DWORD,
    dwRemaining: DWORD,
    pvRef: LPVOID,
) -> BOOL {
    let data: *mut ConfigureDevicesData = pvRef as *mut ConfigureDevicesData;
    let mut device: *mut DeviceData = core::ptr::null_mut();
    let mut i: i32;
    let mut j: i32;

    IDirectInputDevice_AddRef!(lpdid);

    /* alloc array for devices if this is our first device */
    if (*data).devices_data.ndevices == 0 {
        (*data).devices_data.devices = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (core::mem::size_of::<DeviceData>() as SIZE_T)
                .wrapping_mul(dwRemaining.wrapping_add(1 as DWORD) as SIZE_T),
        ) as *mut DeviceData;
    }
    device = (*data)
        .devices_data
        .devices
        .offset((*data).devices_data.ndevices as isize) as *mut DeviceData;
    (*device).lpdid = lpdid;
    (*device).ddi = *lpddi;

    (*device).nobjects = 0 as i32;
    IDirectInputDevice_EnumObjects!(
        lpdid,
        { let cb: LPDIENUMDEVICEOBJECTSCALLBACKW = Some(collect_objects); cb },
        device as LPVOID,
        DIDFT_ALL as DWORD,
    );

    (*device).user_afs = heap_alloc(
        ((core::mem::size_of::<DIACTIONFORMATW>() as SIZE_T)
            .wrapping_mul((*data).nusernames as SIZE_T)) as _,
    ) as *mut DIACTIONFORMATW;
    memset(
        (*device).user_afs as *mut core::ffi::c_void,
        0 as i32,
        (core::mem::size_of::<DIACTIONFORMATW>() as usize)
            .wrapping_mul((*data).nusernames as usize),
    );

    i = 0 as i32;
    while i < (*data).nusernames {
        let user_af: *mut DIACTIONFORMATW =
            (*device).user_afs.offset(i as isize) as *mut DIACTIONFORMATW;
        (*user_af).dwNumActions = (*(*data).original_lpdiaf).dwNumActions;
        (*user_af).guidActionMap = (*(*data).original_lpdiaf).guidActionMap;
        (*user_af).rgoAction = heap_alloc(
            ((core::mem::size_of::<DIACTIONW>() as SIZE_T)
                .wrapping_mul((*(*data).original_lpdiaf).dwNumActions as SIZE_T)) as _,
        ) as LPDIACTIONW;
        memset(
            (*user_af).rgoAction as *mut core::ffi::c_void,
            0 as i32,
            (core::mem::size_of::<DIACTIONW>() as usize)
                .wrapping_mul((*(*data).original_lpdiaf).dwNumActions as usize),
        );

        j = 0 as i32;
        while (j as DWORD) < (*user_af).dwNumActions {
            (*(*user_af).rgoAction.offset(j as isize)).dwSemantic =
                (*(*(*data).original_lpdiaf).rgoAction.offset(j as isize)).dwSemantic;
            (*(*user_af).rgoAction.offset(j as isize)).dwFlags =
                (*(*(*data).original_lpdiaf).rgoAction.offset(j as isize)).dwFlags;
            (*(*user_af).rgoAction.offset(j as isize)).u.lptszActionName =
                (*(*(*data).original_lpdiaf).rgoAction.offset(j as isize))
                    .u
                    .lptszActionName;
            j += 1;
        }

        IDirectInputDevice8_BuildActionMap!(
            lpdid,
            user_af as LPDIACTIONFORMATW,
            *(*data).usernames.offset(i as isize) as LPCWSTR,
            0 as DWORD,
        );
        i += 1;
    }

    (*data).devices_data.ndevices += 1;
    return DIENUM_CONTINUE as BOOL;
}

unsafe extern "C" fn init_listview_columns(dialog: HWND) {
    let mut listColumn: LVCOLUMNW = core::mem::zeroed();
    let mut viewRect: RECT = core::mem::zeroed();
    let mut width: core::ffi::c_int = 0;
    let mut column: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    GetClientRect(GetDlgItem(dialog, (IDC_DEVICEOBJECTSLIST as i32)), &raw mut viewRect);
    width = ((viewRect.right - viewRect.left) / 2) as core::ffi::c_int;

    LoadStringW(DINPUT_instance, IDS_OBJECTCOLUMN, core::ptr::addr_of_mut!(column) as LPWSTR, ARRAY_SIZE!(column) as core::ffi::c_int);
    listColumn.mask = (LVCF_TEXT | LVCF_WIDTH | LVCF_SUBITEM) as UINT;
    listColumn.pszText = core::ptr::addr_of_mut!(column) as *mut WCHAR as LPWSTR;
    listColumn.cchTextMax = lstrlenW(listColumn.pszText as LPCWSTR);
    listColumn.cx = width;

    SendDlgItemMessageW(dialog, (IDC_DEVICEOBJECTSLIST as i32), LVM_INSERTCOLUMNW, 0, &raw mut listColumn as LPARAM);

    LoadStringW(DINPUT_instance, IDS_ACTIONCOLUMN, core::ptr::addr_of_mut!(column) as LPWSTR, ARRAY_SIZE!(column) as core::ffi::c_int);
    listColumn.cx = width;
    listColumn.pszText = core::ptr::addr_of_mut!(column) as *mut WCHAR as LPWSTR;
    listColumn.cchTextMax = lstrlenW(listColumn.pszText as LPCWSTR);

    SendDlgItemMessageW(dialog, (IDC_DEVICEOBJECTSLIST as i32), LVM_INSERTCOLUMNW, 1, &raw mut listColumn as LPARAM);
}

unsafe extern "C" fn lv_get_cur_item(mut dialog: HWND) -> ::core::ffi::c_int {
    return SendDlgItemMessageW(dialog, (IDC_DEVICEOBJECTSLIST as i32), LVM_GETNEXTITEM as UINT, -(1 as ::core::ffi::c_int) as WPARAM, LVNI_SELECTED as LPARAM) as ::core::ffi::c_int;
}

unsafe extern "C" fn lv_get_item_data(mut dialog: HWND, mut index: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut item: LVITEMW = core::mem::zeroed();
    if index < 0 {
        return -1;
    }
    item.mask = LVIF_PARAM as UINT;
    item.iItem = index;
    item.iSubItem = 0;
    SendDlgItemMessageW(
        dialog,
        (IDC_DEVICEOBJECTSLIST as i32),
        LVM_GETITEMW as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(item) as LPARAM,
    );
    return item.lParam as ::core::ffi::c_int;
}

unsafe extern "C" fn lv_set_action(dialog: HWND, item: ::core::ffi::c_int, action: ::core::ffi::c_int, lpdiaf: LPDIACTIONFORMATW) {
    static mut no_action: [u16; 2] = [0x002Du16, 0x0000];
    let mut action_text: *const WCHAR = core::ptr::addr_of!(no_action) as *const WCHAR;
    let mut lvItem: LVITEMW = core::mem::zeroed();

    if item < 0 {
        return;
    }

    if action != -1 {
        action_text = (*(*lpdiaf).rgoAction.offset(action as isize)).u.lptszActionName as *const WCHAR;
    }

    /* Keep the action and text in the listview item */
    lvItem.iItem = item;

    lvItem.mask = LVIF_PARAM;
    lvItem.iSubItem = 0;
    lvItem.lParam = action as LPARAM;

    /* Action index */
    SendDlgItemMessageW(dialog, (IDC_DEVICEOBJECTSLIST as i32), LVM_SETITEMW, 0, core::ptr::addr_of_mut!(lvItem) as LPARAM);

    lvItem.mask = LVIF_TEXT;
    lvItem.iSubItem = 1;
    lvItem.pszText = action_text as *mut WCHAR;
    lvItem.cchTextMax = lstrlenW(lvItem.pszText as LPCWSTR);

    /* Text */
    SendDlgItemMessageW(dialog, (IDC_DEVICEOBJECTSLIST as i32), LVM_SETITEMW, 0, core::ptr::addr_of_mut!(lvItem) as LPARAM);
}

unsafe extern "C" fn get_cur_device(dialog: HWND) -> *mut DeviceData {
    let data: *mut ConfigureDevicesData = GetWindowLongPtrW(dialog, DWLP_USER) as *mut ConfigureDevicesData;
    let sel: i32 = SendDlgItemMessageW(dialog, (IDC_CONTROLLERCOMBO as i32), CB_GETCURSEL, 0, 0) as i32;
    return (*data).devices_data.devices.offset(sel as isize) as *mut DeviceData;
}

unsafe extern "C" fn get_cur_lpdiaf(dialog: HWND) -> *mut DIACTIONFORMATW {
    let data: *mut ConfigureDevicesData =
        GetWindowLongPtrW(dialog, DWLP_USER as i32) as *mut ConfigureDevicesData;
    let controller_sel: i32 = SendDlgItemMessageW(
        dialog,
        (IDC_CONTROLLERCOMBO as i32),
        CB_GETCURSEL as u32,
        0 as WPARAM,
        0 as LPARAM,
    ) as i32;
    let player_sel: i32 = SendDlgItemMessageW(
        dialog,
        (IDC_PLAYERCOMBO as i32),
        CB_GETCURSEL as u32,
        0 as WPARAM,
        0 as LPARAM,
    ) as i32;
    return (*(*data).devices_data.devices.offset(controller_sel as isize))
        .user_afs
        .offset(player_sel as isize);
}

unsafe extern "C" fn get_original_lpdiaf(dialog: HWND) -> *mut DIACTIONFORMATW {
    let data: *mut ConfigureDevicesData = GetWindowLongPtrW(dialog, DWLP_USER as ::core::ffi::c_int) as *mut ConfigureDevicesData;
    return (*data).original_lpdiaf as *mut DIACTIONFORMATW;
}

unsafe extern "C" fn dialog_display_only(dialog: HWND) -> i32 {
    let data: *mut ConfigureDevicesData =
        GetWindowLongPtrW(dialog, DWLP_USER as i32) as *mut ConfigureDevicesData;
    return (*data).display_only;
}

unsafe extern "C" fn init_devices(dialog: HWND, data: *mut ConfigureDevicesData) {
    let mut i: ::core::ffi::c_int;

    /* Collect and insert */
    (*data).devices_data.ndevices = 0;
    IDirectInput8_EnumDevicesBySemantics!(
        (*data).lpDI,
        core::ptr::null_mut(),
        (*data).original_lpdiaf,
        { let cb: LPDIENUMDEVICESBYSEMANTICSCBW = Some(collect_devices); cb },
        data as LPVOID,
        0,
    );

    i = 0;
    while i < (*data).devices_data.ndevices {
        SendDlgItemMessageW(
            dialog,
            (IDC_CONTROLLERCOMBO as i32),
            CB_ADDSTRING,
            0,
            core::ptr::addr_of_mut!((*(*data).devices_data.devices.offset(i as isize)).ddi.tszProductName) as LPARAM,
        );
        i += 1;
    }

    i = 0;
    while i < (*data).nusernames {
        SendDlgItemMessageW(
            dialog,
            (IDC_PLAYERCOMBO as i32),
            CB_ADDSTRING,
            0,
            *(*data).usernames.offset(i as isize) as LPARAM,
        );
        i += 1;
    }
}

unsafe extern "C" fn destroy_data(dialog: HWND) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let data: *mut ConfigureDevicesData =
        GetWindowLongPtrW(dialog, DWLP_USER) as *mut ConfigureDevicesData;
    let devices_data: *mut DIDevicesData = core::ptr::addr_of_mut!((*data).devices_data);

    /* Free the devices */
    while i < (*devices_data).ndevices {
        IDirectInputDevice8_Release!((*(*devices_data).devices.offset(i as isize)).lpdid);
        j = 0;
        while j < (*data).nusernames {
            heap_free(
                (*(*(*devices_data).devices.offset(i as isize))
                    .user_afs
                    .offset(j as isize))
                .rgoAction as *mut core::ffi::c_void,
            );
            j += 1;
        }
        heap_free(
            (*(*devices_data).devices.offset(i as isize)).user_afs as *mut core::ffi::c_void,
        );
        i += 1;
    }

    HeapFree(
        GetProcessHeap(),
        0,
        (*devices_data).devices as *mut core::ffi::c_void,
    );
}

unsafe extern "C" fn fill_device_object_list(mut dialog: HWND) {
    let mut device: *mut DeviceData = get_cur_device(dialog);
    let mut lpdiaf: LPDIACTIONFORMATW = get_cur_lpdiaf(dialog);
    let mut item: LVITEMW = core::mem::zeroed();
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;

    /* Clean the listview */
    SendDlgItemMessageW(
        dialog,
        (IDC_DEVICEOBJECTSLIST as i32),
        LVM_DELETEALLITEMS,
        0,
        0
    );

    /* Add each object */
    i = 0;
    while i < (*device).nobjects {
        let mut ddo_inst: DWORD = 0;
        let mut ddo_type: DWORD = 0;
        let mut action: ::core::ffi::c_int = -1;

        item.mask = (LVIF_TEXT | LVIF_PARAM) as UINT;
        item.iItem = i;
        item.iSubItem = 0;
        item.pszText = core::ptr::addr_of_mut!((*device).ddo[i as usize].tszName) as *mut WCHAR as LPWSTR;
        item.cchTextMax = lstrlenW(item.pszText as LPCWSTR);

        /* Add the item */
        SendDlgItemMessageW(
            dialog,
            (IDC_DEVICEOBJECTSLIST as i32),
            LVM_INSERTITEMW,
            0,
            core::ptr::addr_of_mut!(item) as LPARAM
        );
        ddo_inst = (DIDFT_GETINSTANCE!((*device).ddo[i as usize].dwType) as DWORD);
        ddo_type = (DIDFT_GETTYPE!((*device).ddo[i as usize].dwType) as DWORD);

        /* Search for an assigned action for this device */
        j = 0;
        while (j as DWORD) < (*lpdiaf).dwNumActions {
            let mut af_inst: DWORD = (DIDFT_GETINSTANCE!((*(*lpdiaf).rgoAction.offset(j as isize)).dwObjID) as DWORD);
            let mut af_type: DWORD = (DIDFT_GETTYPE!((*(*lpdiaf).rgoAction.offset(j as isize)).dwObjID) as DWORD);
            if af_type == DIDFT_PSHBUTTON as DWORD {
                af_type = DIDFT_BUTTON as DWORD;
            }
            if af_type == DIDFT_RELAXIS as DWORD {
                af_type = DIDFT_AXIS as DWORD;
            }
            /* NOTE previously compared dwType == dwObjId but default buildActionMap actions
             * were PSHBUTTON and RELAXS and didnt show up on config */
            if IsEqualGUID!(
                core::ptr::addr_of!((*(*lpdiaf).rgoAction.offset(j as isize)).guidInstance),
                core::ptr::addr_of!((*device).ddi.guidInstance),
            ) != false
                && ddo_inst == af_inst
                && (ddo_type & af_type) != 0
            {
                action = j;
                break;
            }
            j += 1;
        }

        lv_set_action(dialog, i, action, lpdiaf);
        i += 1;
    }
}

unsafe extern "C" fn show_suitable_actions(mut dialog: HWND) {
    let mut device: *mut DeviceData = get_cur_device(dialog);
    let mut lpdiaf: LPDIACTIONFORMATW = get_original_lpdiaf(dialog) as LPDIACTIONFORMATW;
    let mut i: ::core::ffi::c_int = 0;
    let mut added: ::core::ffi::c_int = 0;
    let mut obj: ::core::ffi::c_int = lv_get_cur_item(dialog);
    if obj < 0 {
        return;
    }
    SendDlgItemMessageW(
        dialog,
        (IDC_ACTIONLIST as i32),
        LB_RESETCONTENT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );
    i = 0;
    while (i as DWORD) < (*lpdiaf).dwNumActions {
        if !(GET_DIDEVICE_TYPE!((*device).ddi.dwDevType) != (DI8DEVTYPE_KEYBOARD as u8)
            && ((*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic & DIKEYBOARD_MASK)
                == DIKEYBOARD_MASK)
        {
            if !(GET_DIDEVICE_TYPE!((*device).ddi.dwDevType) != (DI8DEVTYPE_MOUSE as u8)
                && ((*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic & DIMOUSE_MASK)
                    == DIMOUSE_MASK)
            {
                if ((DIDFT_GETINSTANCE!((*(*lpdiaf).rgoAction.offset(i as isize)).dwSemantic) as DWORD)
                    & (DIDFT_GETTYPE!((*device).ddo[obj as usize].dwType) as DWORD))
                    != 0
                {
                    SendDlgItemMessageW(
                        dialog,
                        IDC_ACTIONLIST as ::core::ffi::c_int,
                        LB_ADDSTRING as UINT,
                        0 as WPARAM,
                        (*(*lpdiaf).rgoAction.offset(i as isize)).u.lptszActionName as LPARAM,
                    );
                    SendDlgItemMessageW(
                        dialog,
                        IDC_ACTIONLIST as ::core::ffi::c_int,
                        LB_SETITEMDATA as UINT,
                        added as WPARAM,
                        i as LPARAM,
                    );
                    added += 1;
                }
            }
        }
        i += 1;
    }
}

unsafe extern "C" fn assign_action(dialog: HWND) {
    let device: *mut DeviceData = get_cur_device(dialog);
    let lpdiaf: LPDIACTIONFORMATW = get_cur_lpdiaf(dialog);
    let mut lvFind: LVFINDINFOW = core::mem::zeroed();
    let sel: ::core::ffi::c_int = SendDlgItemMessageW(
        dialog,
        IDC_ACTIONLIST as ::core::ffi::c_int,
        LB_GETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as ::core::ffi::c_int;
    let action: ::core::ffi::c_int = SendDlgItemMessageW(
        dialog,
        IDC_ACTIONLIST as ::core::ffi::c_int,
        LB_GETITEMDATA as UINT,
        sel as WPARAM,
        0 as LPARAM,
    ) as ::core::ffi::c_int;
    let obj: ::core::ffi::c_int = lv_get_cur_item(dialog);
    let old_action: ::core::ffi::c_int = lv_get_item_data(dialog, obj);
    let mut used_obj: ::core::ffi::c_int;
    let mut type_0: DWORD;

    if old_action == action {
        return;
    }
    if obj < 0 as ::core::ffi::c_int {
        return;
    }
    if ((*(*lpdiaf).rgoAction.offset(old_action as isize)).dwFlags
        & (DIA_APPFIXED as DWORD))
        != 0
    {
        return;
    }

    type_0 = (*device).ddo[obj as usize].dwType;

    /* Clear old action */
    if old_action != -(1 as ::core::ffi::c_int) {
        (*(*lpdiaf).rgoAction.offset(old_action as isize)).dwObjID =
            0 as DWORD;
        (*(*lpdiaf).rgoAction.offset(old_action as isize)).guidInstance =
            GUID_NULL as GUID;
        (*(*lpdiaf).rgoAction.offset(old_action as isize)).dwHow =
            DIAH_UNMAPPED as DWORD;
    }

    /* Find if action text is already set for other object and unset it */
    lvFind.flags = LVFI_PARAM as UINT;
    lvFind.lParam = action as LPARAM;

    used_obj = SendDlgItemMessageW(
        dialog,
        IDC_DEVICEOBJECTSLIST as ::core::ffi::c_int,
        LVM_FINDITEMW as UINT,
        (-(1 as ::core::ffi::c_int)) as WPARAM,
        ::core::ptr::addr_of_mut!(lvFind) as LPARAM,
    ) as ::core::ffi::c_int;

    lv_set_action(
        dialog,
        used_obj,
        -(1 as ::core::ffi::c_int),
        lpdiaf,
    );

    /* Set new action */
    (*(*lpdiaf).rgoAction.offset(action as isize)).dwObjID = type_0;
    (*(*lpdiaf).rgoAction.offset(action as isize)).guidInstance = (*device).ddi.guidInstance;
    (*(*lpdiaf).rgoAction.offset(action as isize)).dwHow =
        DIAH_USERCONFIG as DWORD;

    /* Set new action in the list */
    lv_set_action(dialog, obj, action, lpdiaf);
}

unsafe extern "C" fn reset_actions(dialog: HWND) {
    let data: *mut ConfigureDevicesData =
        GetWindowLongPtrW(dialog, DWLP_USER) as *mut ConfigureDevicesData;
    let ddata: *mut DIDevicesData = core::ptr::addr_of_mut!((*data).devices_data);
    let mut i: u32;
    let mut j: u32;

    i = 0;
    while i < (*data).devices_data.ndevices as u32 {
        let device: *mut DeviceData = (*ddata).devices.add(i as usize);
        j = 0;
        while j < (*data).nusernames as u32 {
            IDirectInputDevice8_BuildActionMap!(
                (*device).lpdid,
                (*device).user_afs.add(j as usize),
                *(*data).usernames.add(j as usize),
                DIDBAM_HWDEFAULTS,
            );
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn save_actions(dialog: HWND) {
    let mut data: *mut ConfigureDevicesData =
        GetWindowLongPtrW(dialog, DWLP_USER as ::core::ffi::c_int) as *mut ConfigureDevicesData;
    let mut ddata: *mut DIDevicesData = &raw mut (*data).devices_data;
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;
    if (*data).display_only == 0 {
        i = 0 as ::core::ffi::c_uint;
        while i < (*ddata).ndevices as ::core::ffi::c_uint {
            let mut device: *mut DeviceData =
                (*ddata).devices.offset(i as isize) as *mut DeviceData;
            j = 0 as ::core::ffi::c_uint;
            while j < (*data).nusernames as ::core::ffi::c_uint {
                if save_mapping_settings(
                    (*device).lpdid,
                    (*device).user_afs.offset(j as isize) as LPDIACTIONFORMATW,
                    *(*data).usernames.offset(j as isize) as LPCWSTR,
                ) != DI_OK
                {
                    MessageBoxA(
                        dialog,
                        b"Could not save settings\0".as_ptr() as LPCSTR,
                        core::ptr::null(),
                        MB_ICONERROR,
                    );
                }
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
    }
}

unsafe extern "system" fn ConfigureDevicesDlgProc(
    dialog: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            let data: *mut ConfigureDevicesData = lParam as *mut ConfigureDevicesData;

            /* Initialize action format and enumerate devices */
            init_devices(dialog, data);

            /* Store information in the window */
            SetWindowLongPtrW(
                dialog,
                DWLP_USER as ::core::ffi::c_int,
                data as LONG_PTR,
            );

            init_listview_columns(dialog);

            /* Select the first device and show its actions */
            SendDlgItemMessageW(
                dialog,
                IDC_CONTROLLERCOMBO as ::core::ffi::c_int,
                CB_SETCURSEL as UINT,
                0 as WPARAM,
                0 as LPARAM,
            );
            SendDlgItemMessageW(
                dialog,
                IDC_PLAYERCOMBO as ::core::ffi::c_int,
                CB_SETCURSEL as UINT,
                0 as WPARAM,
                0 as LPARAM,
            );
            fill_device_object_list(dialog);

            ShowCursor((TRUE) as BOOL);
        }
        WM_DESTROY => {
            ShowCursor((FALSE) as BOOL);
        }
        WM_NOTIFY => match ((*(lParam as LPNMHDR)).code) as i32 {
            LVN_ITEMCHANGED => {
                show_suitable_actions(dialog);
            }
            _ => {}
        },
        WM_COMMAND => match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
            IDC_ACTIONLIST => {
                match (HIWORD!(wParam) as ::core::ffi::c_int) as u32 {
                    LBN_DBLCLK => {
                        /* Ignore this if app did not ask for editing */
                        if dialog_display_only(dialog) == 0 {
                            assign_action(dialog);
                        }
                    }
                    _ => {}
                }
            }
            IDC_CONTROLLERCOMBO | IDC_PLAYERCOMBO => {
                match (HIWORD!(wParam) as ::core::ffi::c_int) as u32 {
                    CBN_SELCHANGE => {
                        fill_device_object_list(dialog);
                    }
                    _ => {}
                }
            }
            IDOK => {
                save_actions(dialog);
                EndDialog(dialog, 0 as INT_PTR);
                destroy_data(dialog);
            }
            IDCANCEL => {
                EndDialog(dialog, 0 as INT_PTR);
                destroy_data(dialog);
            }
            IDC_RESET => {
                reset_actions(dialog);
                fill_device_object_list(dialog);
            }
            _ => {}
        },
        _ => {}
    }

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn _configure_devices(
    mut iface: *mut IDirectInput8W,
    mut lpdiCallback: LPDICONFIGUREDEVICESCALLBACK,
    mut lpdiCDParams: LPDICONFIGUREDEVICESPARAMSW,
    mut dwFlags: DWORD,
    mut pvRefData: LPVOID,
) -> HRESULT {
    let mut i: ::core::ffi::c_int = 0;
    let mut size: DWORD = 0;
    let mut username: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut data: ConfigureDevicesData = ConfigureDevicesData {
        lpDI: ::core::ptr::null_mut::<IDirectInput8W>(),
        original_lpdiaf: ::core::ptr::null_mut::<_DIACTIONFORMATW>(),
        devices_data: DIDevicesData {
            ndevices: 0,
            devices: ::core::ptr::null_mut::<DeviceData>(),
        },
        display_only: 0,
        nusernames: 0,
        usernames: ::core::ptr::null_mut::<*mut WCHAR>(),
    };
    data.lpDI = iface;
    data.original_lpdiaf = (*lpdiCDParams).lprgFormats;
    data.display_only = ((dwFlags & (DICD_EDIT as DWORD)) == 0) as ::core::ffi::c_int;
    data.nusernames = (*lpdiCDParams).dwcUsers as ::core::ffi::c_int;
    if (*lpdiCDParams).lptszUserNames.is_null() {
        GetUserNameW(::core::ptr::null_mut::<WCHAR>(), &raw mut size);
        username = heap_alloc(
            ((size as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T)) as _,
        ) as *mut WCHAR;
        GetUserNameW(username as LPWSTR, &raw mut size);
        data.nusernames = 1 as ::core::ffi::c_int;
        data.usernames = heap_alloc(
            (::core::mem::size_of::<*mut WCHAR>() as SIZE_T) as _,
        ) as *mut *mut WCHAR;
        *data.usernames.offset(0 as isize) = username;
    } else {
        let mut p: *mut WCHAR = (*lpdiCDParams).lptszUserNames as *mut WCHAR;
        data.usernames = heap_alloc(
            ((::core::mem::size_of::<*mut WCHAR>() as SIZE_T)
                .wrapping_mul(data.nusernames as SIZE_T)) as _,
        ) as *mut *mut WCHAR;
        i = 0 as ::core::ffi::c_int;
        while i < data.nusernames {
            if *p != 0 {
                *data.usernames.offset(i as isize) = p;
                loop {
                    let fresh17 = p;
                    p = p.offset(1);
                    if !(*fresh17 != 0) {
                        break;
                    }
                }
            } else {
                return DIERR_INVALIDPARAM;
            }
            i += 1;
        }
    }

    InitCommonControls();

    DialogBoxParamW(
        DINPUT_instance,
        MAKEINTRESOURCE!(IDD_CONFIGUREDEVICES) as LPCWSTR,
        (*lpdiCDParams).hwnd,
        Some(
            ConfigureDevicesDlgProc,
        ),
        &raw mut data as LPARAM,
    );

    heap_free(username as *mut ::core::ffi::c_void);
    heap_free(data.usernames as *mut ::core::ffi::c_void);

    return DI_OK;
}
