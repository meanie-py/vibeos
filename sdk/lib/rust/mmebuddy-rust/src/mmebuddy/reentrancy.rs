use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut EntrypointMutexes: [HANDLE; 6] =
    [::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void; 6];

// SOUND_DEVICE_TYPES is not defined in this preamble; the value 3 is assumed and must match the C header.
pub const SOUND_DEVICE_TYPES: usize = 3;
#[no_mangle]
pub unsafe extern "C" fn InitEntrypointMutexes() -> MMRESULT {
    let mut i: UCHAR = 0 as UCHAR;
    let mut Result: MMRESULT = MMSYSERR_NOERROR as MMRESULT;

    /* Blank all entries ni the table first */
    i = 0 as UCHAR;
    while (i as ::core::ffi::c_int) < SOUND_DEVICE_TYPES as ::core::ffi::c_int {
        EntrypointMutexes[i as usize] = core::ptr::null_mut();
        i = i.wrapping_add(1);
    }

    /* Now create the mutexes */
    i = 0 as UCHAR;
    while (i as ::core::ffi::c_int) < SOUND_DEVICE_TYPES as ::core::ffi::c_int {
        EntrypointMutexes[i as usize] = CreateMutex(
            core::ptr::null_mut(),
            FALSE as _,
            core::ptr::null_mut(),
        );

        if EntrypointMutexes[i as usize].is_null() {
            Result = Win32ErrorToMmResult(GetLastError() as UINT);

            /* Clean up any mutexes we successfully created */
            CleanupEntrypointMutexes();
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }

    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn CleanupEntrypointMutexes() {
    let mut i: UCHAR = 0;

    /* Only clean up a mutex if it actually exists */
    while (i as ::core::ffi::c_int) < SOUND_DEVICE_TYPES as ::core::ffi::c_int {
        if !EntrypointMutexes[i as usize].is_null() {
            CloseHandle(EntrypointMutexes[i as usize]);
            EntrypointMutexes[i as usize] = core::ptr::null_mut();
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AcquireEntrypointMutex(DeviceType: MMDEVICE_TYPE) {
    let mut i: UCHAR;

    SND_ASSERT!(IS_VALID_SOUND_DEVICE_TYPE!(DeviceType));
    i = (SOUND_DEVICE_TYPE_TO_INDEX!(DeviceType)) as u8;

    SND_ASSERT!(EntrypointMutexes[i as usize]);

    WaitForSingleObject(EntrypointMutexes[i as usize], INFINITE);
}
#[no_mangle]
pub unsafe extern "C" fn ReleaseEntrypointMutex(DeviceType: MMDEVICE_TYPE) {
    let mut i: UCHAR;

    SND_ASSERT!(IS_VALID_SOUND_DEVICE_TYPE!(DeviceType));
    i = (SOUND_DEVICE_TYPE_TO_INDEX!(DeviceType)) as u8;

    SND_ASSERT!(EntrypointMutexes[i as usize]);

    ReleaseMutex(EntrypointMutexes[i as usize]);
}
