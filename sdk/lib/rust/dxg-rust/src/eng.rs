//! win32ss/reactx/dxg/eng.c

use win32k_sys::win32k::*;
use crate::types::*;
use crate::ddhmg::DdHmgLock;

#[no_mangle]
pub unsafe extern "system" fn DxDdLockDirectDrawSurface(hDdSurface: HANDLE) -> PDD_SURFACE_LOCAL {
    let mut pSurfacelcl: PDD_SURFACE_LOCAL = core::ptr::null_mut();

    let pEDDSurface = DdHmgLock(hDdSurface, ObjType_DDSURFACE_TYPE, 0) as PEDD_SURFACE;
    if !pEDDSurface.is_null() {
        pSurfacelcl = core::ptr::addr_of_mut!((*pEDDSurface).ddsSurfaceLocal);
    }

    pSurfacelcl
}

#[no_mangle]
pub unsafe extern "system" fn DxDdUnlockDirectDrawSurface(pSurface: PDD_SURFACE_LOCAL) -> BOOL {
    let mut retVal: BOOL = 0;

    if !pSurface.is_null() {
        retVal = 1;
    }

    retVal
}
