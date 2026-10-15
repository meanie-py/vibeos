use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn DCompositionCreateDevice(
    dxgi_device: *mut IDXGIDevice,
    iid: REFIID,
    device: *mut *mut core::ffi::c_void,
) -> HRESULT {
    // FIXME("%p, %s, %p.\n", dxgi_device, debugstr_guid(iid), device);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn DCompositionCreateDevice2(
    rendering_device: *mut IUnknown,
    iid: REFIID,
    device: *mut *mut core::ffi::c_void,
) -> HRESULT {
    // FIXME("%p, %s, %p.\n", rendering_device, debugstr_guid(iid), device);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn DCompositionCreateDevice3(
    rendering_device: *mut IUnknown,
    iid: REFIID,
    device: *mut *mut core::ffi::c_void,
) -> HRESULT {
    // FIXME("%p, %s, %p.\n", rendering_device, debugstr_guid(iid), device);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn DCompositionCreateSurfaceHandle(access: DWORD, attributes: *mut SECURITY_ATTRIBUTES, handle: *mut HANDLE) -> HRESULT {
    FIXME!("%#lx, %p, %p stub!\n", access, attributes, handle);
    if !handle.is_null() {
        *handle = core::ptr::null_mut();
    }
    return E_NOTIMPL;
}
