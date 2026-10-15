use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn Tbsi_Context_Create(
    params: *const TBS_CONTEXT_PARAMS,
    out: *mut TBS_HCONTEXT,
) -> TBS_RESULT {
    // FIXME("(%p, %p) stub\n", params, out);
    (TBS_E_TPM_NOT_FOUND) as u32
}

#[no_mangle]
pub unsafe extern "system" fn Tbsi_GetDeviceInfo(
    size: UINT32,
    info: *mut core::ffi::c_void,
) -> TBS_RESULT {
    // FIXME("(%u, %p) stub\n", size, info);
    (TBS_E_TPM_NOT_FOUND) as u32
}

#[no_mangle]
pub unsafe extern "system" fn GetDeviceIDString(
    out: *mut WCHAR,
    size: UINT32,
    used: *mut UINT32,
    tpm: *mut BOOL,
) -> HRESULT {
    // FIXME("(%p, %u, %p, %p) stub\n", out, size, used, tpm);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn Tbsip_Context_Close(context: TBS_HCONTEXT) -> TBS_RESULT {
    // FIXME("(%p) stub\n", context);
    (TBS_E_INVALID_CONTEXT) as u32
}

#[no_mangle]
pub unsafe extern "system" fn Tbsip_Cancel_Commands(context: TBS_HCONTEXT) -> TBS_RESULT {
    // FIXME("(%p) stub\n", context);
    (TBS_E_INVALID_CONTEXT) as u32
}

#[no_mangle]
pub unsafe extern "system" fn Tbsip_Submit_Command(
    context: TBS_HCONTEXT,
    locality: TBS_COMMAND_LOCALITY,
    priority: TBS_COMMAND_PRIORITY,
    command: *const BYTE,
    command_size: UINT32,
    result: *mut BYTE,
    result_size: *mut UINT32,
) -> TBS_RESULT {
    // FIXME("(%p, %u, %u, %p, %u, %p, %p) stub\n", context, locality, priority, command, command_size, result, result_size);
    (TBS_E_INVALID_CONTEXT) as u32
}
