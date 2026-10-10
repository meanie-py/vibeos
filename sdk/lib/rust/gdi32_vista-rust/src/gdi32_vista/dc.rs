use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn D3DKMTCreateDCFromMemory(
    desc: *mut D3DKMT_CREATEDCFROMMEMORY,
) -> NTSTATUS {
    (NtGdiDdDDICreateDCFromMemory(desc)) as i32
}

#[no_mangle]
pub unsafe extern "system" fn D3DKMTDestroyDCFromMemory(
    desc: *const D3DKMT_DESTROYDCFROMMEMORY,
) -> NTSTATUS {
    (NtGdiDdDDIDestroyDCFromMemory(desc)) as i32
}
