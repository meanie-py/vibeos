use crate::audiosrv::pnp_list_manager::AppendAudioDeviceToList;
use crate::audiosrv::pnp_list_manager::CreateDeviceDescriptor;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_FB213D92F063: [u16; 19] = [
    0x0077u16, 0x0069, 0x006E, 0x006D, 0x006D, 0x005F, 0x0064, 0x0065, 0x0076, 0x0069,
    0x0063, 0x0065, 0x0063, 0x0068, 0x0061, 0x006E, 0x0067, 0x0065, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut device_notification_handle: HDEVNOTIFY =
    ::core::ptr::null::<HDEVNOTIFY__>() as *mut HDEVNOTIFY__;

static mut uWinMMDeviceChange: UINT = 0 as UINT;
#[no_mangle]
pub unsafe extern "C" fn ProcessExistingDevices() -> BOOL {
    let mut interface_data: SP_DEVICE_INTERFACE_DATA = core::mem::zeroed();
    let mut device_data: SP_DEVINFO_DATA = core::mem::zeroed();
    let mut detail_data: PSP_DEVICE_INTERFACE_DETAIL_DATA_W = core::ptr::null_mut();
    let mut dev_info: HDEVINFO = core::ptr::null_mut();
    let mut length: DWORD = 0;
    let mut index: core::ffi::c_int = 0;

    let category_guid: GUID = STATIC_KSCATEGORY_AUDIO;

    dev_info = SetupDiGetClassDevsExW(
        &raw const category_guid,
        core::ptr::null::<WCHAR>(),
        core::ptr::null_mut::<HWND__>(),
        (DIGCF_PRESENT | DIGCF_DEVICEINTERFACE) as DWORD,
        core::ptr::null_mut(),
        core::ptr::null::<WCHAR>(),
        core::ptr::null_mut(),
    );

    interface_data.cbSize = core::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as DWORD;
    interface_data.Reserved = 0 as ULONG_PTR;

    /* Enumerate the devices within the category */
    index = 0;

    length = (core::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA>() as usize)
        .wrapping_add((MAX_PATH as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize))
        as DWORD;

    detail_data = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        length as SIZE_T,
    ) as PSP_DEVICE_INTERFACE_DETAIL_DATA_W;

    if detail_data.is_null() {
        DPRINT!(b"failed to allocate detail_data\n\0".as_ptr() as *const core::ffi::c_char);
        return TRUE as BOOL;
    }

    while SetupDiEnumDeviceInterfaces(
        dev_info,
        core::ptr::null_mut(),
        &raw const category_guid,
        index as DWORD,
        &raw mut interface_data,
    ) != 0
    {
        let mut list_node: *mut PnP_AudioDevice = core::ptr::null_mut();

        ZeroMemory!(detail_data as *mut core::ffi::c_void, length as usize);

        /* NOTE: We don't actually use device_data... */
        (*detail_data).cbSize =
            core::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as DWORD;
        device_data.cbSize = core::mem::size_of::<SP_DEVINFO_DATA>() as DWORD;
        device_data.Reserved = 0 as ULONG_PTR;
        SetupDiGetDeviceInterfaceDetailW(
            dev_info,
            &raw mut interface_data,
            detail_data,
            length,
            core::ptr::null_mut(),
            &raw mut device_data,
        );

        list_node = CreateDeviceDescriptor(
            &raw mut (*detail_data).DevicePath as *mut WCHAR,
            TRUE as BOOL,
        ) as *mut PnP_AudioDevice;
        AppendAudioDeviceToList(list_node);
        DestroyDeviceDescriptor!(list_node);

        /* TODO: Cleanup the device we enumerated? */

        index += 1;
    }

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        detail_data as PVOID,
    );

    SetupDiDestroyDeviceInfoList(dev_info);

    uWinMMDeviceChange = RegisterWindowMessageW(RUSTCONV_WSTR_FB213D92F063.as_ptr());

    if uWinMMDeviceChange == 0 {
        DPRINT1!(
            b"Failed to register winmm_devicechange window message: GetLastError %u\n\0".as_ptr()
                as *const core::ffi::c_char,
            GetLastError(),
        );
    }

    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ProcessDeviceArrival(device: *mut DEV_BROADCAST_DEVICEINTERFACE) -> DWORD {
    let mut list_node: *mut PnP_AudioDevice = core::ptr::null_mut();
    list_node = CreateDeviceDescriptor(
        core::ptr::addr_of_mut!((*device).dbcc_name) as *mut WCHAR,
        TRUE as BOOL,
    ) as *mut PnP_AudioDevice;
    AppendAudioDeviceToList(list_node);
    DestroyDeviceDescriptor!(list_node);

    let mut dwInfo: DWORD = BSM_ALLDESKTOPS | BSM_APPLICATIONS;
    BroadcastSystemMessageW(
        BSF_POSTMESSAGE,
        core::ptr::addr_of_mut!(dwInfo),
        uWinMMDeviceChange,
        0,
        0,
    );

    return NO_ERROR;
}
#[no_mangle]
pub unsafe extern "C" fn ProcessDeviceRemovalComplete(device: *mut DEV_BROADCAST_DEVICEINTERFACE) -> DWORD {
    /* TODO: clean up appended audio devices */
    let mut dwInfo: DWORD = (BSM_ALLDESKTOPS as DWORD) | (BSM_APPLICATIONS as DWORD);
    BroadcastSystemMessageW(
        BSF_POSTMESSAGE as DWORD,
        core::ptr::addr_of_mut!(dwInfo),
        uWinMMDeviceChange,
        0 as WPARAM,
        0 as LPARAM,
    );

    return NO_ERROR as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn RegisterForDeviceNotifications() -> BOOL {
    let mut notification_filter: DEV_BROADCAST_DEVICEINTERFACE = core::mem::zeroed();
    let wdmaud_guid: GUID = STATIC_KSCATEGORY_AUDIO;
    ZeroMemory!(
        core::ptr::addr_of_mut!(notification_filter) as *mut core::ffi::c_void,
        core::mem::size_of::<DEV_BROADCAST_DEVICEINTERFACE>(),
    );
    notification_filter.dbcc_size =
        core::mem::size_of::<DEV_BROADCAST_DEVICEINTERFACE>() as DWORD;
    notification_filter.dbcc_devicetype = DBT_DEVTYP_DEVICEINTERFACE as DWORD;
    notification_filter.dbcc_classguid = wdmaud_guid;

    device_notification_handle = RegisterDeviceNotificationW(
        service_status_handle as HANDLE,
        core::ptr::addr_of_mut!(notification_filter) as LPVOID,
        DEVICE_NOTIFY_SERVICE_HANDLE as DWORD,
    );
    if device_notification_handle.is_null() {
        DPRINT!(
            b"failed with error %d\n\0".as_ptr() as *const i8,
            GetLastError(),
        );
    }

    return (device_notification_handle != core::ptr::null_mut()) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn UnregisterDeviceNotifications() {
    if !device_notification_handle.is_null() {
        if UnregisterDeviceNotification(device_notification_handle) != 0 {
            device_notification_handle = core::ptr::null_mut();
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn HandleDeviceEvent(
    dwEventType: DWORD,
    lpEventData: LPVOID,
) -> DWORD {
    match dwEventType {
        DBT_DEVICEARRIVAL => {
            let incoming_device: *mut DEV_BROADCAST_DEVICEINTERFACE =
                lpEventData as *mut DEV_BROADCAST_DEVICEINTERFACE;

            return ProcessDeviceArrival(incoming_device);
        }

        DBT_DEVICEREMOVECOMPLETE => {
            let removed_device: *mut DEV_BROADCAST_DEVICEINTERFACE =
                lpEventData as *mut DEV_BROADCAST_DEVICEINTERFACE;

            return ProcessDeviceRemovalComplete(removed_device);
        }

        _ => {
            // break
        }
    }

    return NO_ERROR;
}
