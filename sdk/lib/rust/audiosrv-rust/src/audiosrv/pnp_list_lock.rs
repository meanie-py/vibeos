use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const AUDIO_LIST_LOCK_NAME: [::core::ffi::c_ushort; 27] = unsafe {
    ::core::mem::transmute::<[u8; 54], [::core::ffi::c_ushort; 27]>(
        *b"G\0l\0o\0b\0a\0l\0\\\0A\0u\0d\0i\0o\0D\0e\0v\0i\0c\0e\0L\0i\0s\0t\0L\0o\0c\0k\0\0\0",
    )
};

static mut audio_device_list_lock: HANDLE = NULL;
#[no_mangle]
pub unsafe extern "C" fn InitializeAudioDeviceListLock() -> BOOL {
    let mut security_descriptor: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut security: SECURITY_ATTRIBUTES = core::mem::zeroed();

    InitializeSecurityDescriptor(core::ptr::addr_of_mut!(security_descriptor) as PSECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR_REVISION);
    SetSecurityDescriptorDacl(core::ptr::addr_of_mut!(security_descriptor) as PSECURITY_DESCRIPTOR, TRUE as BOOL, core::ptr::null_mut(), FALSE as BOOL);

    security.nLength = core::mem::size_of::<SECURITY_ATTRIBUTES>() as DWORD;
    security.lpSecurityDescriptor = core::ptr::addr_of_mut!(security_descriptor) as LPVOID;
    security.bInheritHandle = FALSE as BOOL;

    audio_device_list_lock = CreateMutex(core::ptr::addr_of_mut!(security),
                                         FALSE as BOOL,
                                         AUDIO_LIST_LOCK_NAME.as_ptr());

    return (audio_device_list_lock != core::ptr::null_mut()) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn KillAudioDeviceListLock() {
    CloseHandle(audio_device_list_lock);
    audio_device_list_lock = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn LockAudioDeviceList() {
    assert!(!audio_device_list_lock.is_null());
    WaitForSingleObject(audio_device_list_lock, INFINITE);
}
#[no_mangle]
pub unsafe extern "C" fn UnlockAudioDeviceList() {
    assert!(!audio_device_list_lock.is_null());
    ReleaseMutex(audio_device_list_lock);
}
