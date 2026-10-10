use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const AUDIO_LIST_NAME: [::core::ffi::c_ushort; 23] = unsafe {
    ::core::mem::transmute::<[u8; 46], [::core::ffi::c_ushort; 23]>(
        *b"G\0l\0o\0b\0a\0l\0\\\0A\0u\0d\0i\0o\0D\0e\0v\0i\0c\0e\0L\0i\0s\0t\0\0\0",
    )
};

static mut device_list_file: HANDLE = NULL;

static mut audio_device_list: *mut PnP_AudioHeader =
    ::core::ptr::null::<PnP_AudioHeader>() as *mut PnP_AudioHeader;
#[no_mangle]
pub unsafe extern "C" fn CreateDeviceDescriptor(path: *mut WCHAR, is_enabled: BOOL) -> *mut c_void {
    let device: *mut PnP_AudioDevice;

    let path_length: i32 = WideStringSize!(path) as i32;
    let size: i32 = core::mem::size_of::<PnP_AudioDevice>() as i32 + path_length;

    device = malloc(size as usize) as *mut PnP_AudioDevice;
    if device.is_null() {
        DPRINT!("Failed to malloc device descriptor\n");
        return core::ptr::null_mut();
    }

    (*device).enabled = is_enabled as DWORD;
    memcpy(
        core::ptr::addr_of_mut!((*device).path) as *mut c_void,
        path as *const c_void,
        path_length as usize,
    );

    device as *mut c_void
}
#[no_mangle]
pub unsafe extern "C" fn AppendAudioDeviceToList(device: *mut PnP_AudioDevice) -> BOOL {
    let mut device_info_size: i32;

    /* Figure out the actual structure size */
    device_info_size = core::mem::size_of::<PnP_AudioDevice>() as i32;
    device_info_size += WideStringSize!(core::ptr::addr_of_mut!((*device).path) as *mut WCHAR) as i32;

    LockAudioDeviceList();

    /* We DON'T want to overshoot the end of the buffer! */
    if (*audio_device_list).size.wrapping_add(device_info_size as DWORD) > (*audio_device_list).max_size
    {
        /*DPRINT("failed, max_size would be exceeded\n");*/

        UnlockAudioDeviceList();

        return FALSE as BOOL;
    }

    /* Commit the device descriptor to the list */
    memcpy(
        (audio_device_list as *mut core::ffi::c_char).offset((*audio_device_list).size as isize)
            as *mut core::ffi::c_void,
        device as *const core::ffi::c_void,
        device_info_size as usize,
    );

    /* Update the header */
    (*audio_device_list).device_count = (*audio_device_list).device_count.wrapping_add(1);
    (*audio_device_list).size = (*audio_device_list).size.wrapping_add(device_info_size as DWORD);

    UnlockAudioDeviceList();

    DPRINT!(b"Device added to list\n\0".as_ptr() as *const i8);

    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn CreateAudioDeviceList(max_size: DWORD) -> BOOL {
    if InitializeAudioDeviceListLock() == 0 {
        /*DPRINT("Failed\n");*/
        return FALSE as BOOL;
    }

    /* Preliminary locking - the list memory will likely be a big
       buffer of gibberish at this point so we don't want anyone
       turning up before we're ready... */
    LockAudioDeviceList();

    // DPRINT("Creating file mapping\n");
    /* Expose our device list to the world */
    device_list_file = CreateFileMappingW(
        INVALID_HANDLE_VALUE,
        core::ptr::null_mut(),
        PAGE_READWRITE,
        0,
        max_size,
        AUDIO_LIST_NAME.as_ptr(),
    );
    if device_list_file.is_null() {
        // DPRINT("Creation of audio device list failed (err %d)\n", GetLastError());
        let _last_err = GetLastError();

        UnlockAudioDeviceList();
        KillAudioDeviceListLock();

        return FALSE as BOOL;
    }

    // DPRINT("Mapping view of file\n");
    /* Of course, we'll need to access the list ourselves */
    audio_device_list = MapViewOfFile(
        device_list_file,
        FILE_MAP_WRITE,
        0,
        0,
        max_size as SIZE_T,
    ) as *mut PnP_AudioHeader;
    if audio_device_list.is_null() {
        // DPRINT("MapViewOfFile FAILED (err %d)\n", GetLastError());
        let _last_err = GetLastError();

        CloseHandle(device_list_file);
        device_list_file = core::ptr::null_mut();

        UnlockAudioDeviceList();
        KillAudioDeviceListLock();

        return FALSE as BOOL;
    }

    /* Clear the mem to avoid any random stray data */
    memset(
        audio_device_list as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        max_size as usize,
    );

    /* Don't want devices to overwrite the list! */
    (*audio_device_list).size = core::mem::size_of::<PnP_AudioHeader>() as DWORD;
    (*audio_device_list).max_size = max_size;
    (*audio_device_list).device_count = 0 as DWORD;

    UnlockAudioDeviceList();

    // DPRINT("Device list created\n");

    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn DestroyAudioDeviceList() {
    DPRINT!(b"Destroying device list\n\0".as_ptr() as *const i8);

    LockAudioDeviceList();

    /*DPRINT("Unmapping view\n");*/
    UnmapViewOfFile(audio_device_list as LPCVOID);
    audio_device_list = core::ptr::null_mut();

    /*DPRINT("Closing memory mapped file\n");*/
    CloseHandle(device_list_file);
    device_list_file = core::ptr::null_mut();

    UnlockAudioDeviceList();

    /*DPRINT("Killing devlist lock\n");*/
    KillAudioDeviceListLock();
}
