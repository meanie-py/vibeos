//! win32ss/reactx/dxg/dd.c

use win32k_sys::win32k::*;
use ntkrnl_sys::interlocked::InterlockedDecrement;
use crate::types::*;
use crate::ddhmg::DdHmgLock;
use crate::driver_main::gpEngFuncs;

#[no_mangle]
pub unsafe extern "system" fn DxDdCanCreateSurface(
    DdHandle: HANDLE,
    SurfaceData: PDD_CANCREATESURFACEDATA,
) -> DWORD {
    let peDdL: PEDD_DIRECTDRAW_LOCAL;
    let peDdGl: PEDD_DIRECTDRAW_GLOBAL;
    let mut RetVal: DWORD = DDHAL_DRIVER_NOTHANDLED;

    peDdL = DdHmgLock(DdHandle, ObjType_DDLOCAL_TYPE, 0) as PEDD_DIRECTDRAW_LOCAL;
    if peDdL.is_null() {
        return RetVal;
    }

    peDdGl = (*peDdL).peDirectDrawGlobal2;
    (gpEngFuncs.DxEngLockHdev.unwrap())((*peDdGl).hDev);

    (*SurfaceData).lpDD = peDdGl as PDD_DIRECTDRAW_GLOBAL;

    if ((*peDdGl).ddCallbacks.dwFlags & DDHAL_CB32_CANCREATESURFACE) != 0 {
        RetVal = ((*peDdGl).ddCallbacks.CanCreateSurface.unwrap())(SurfaceData);
    }

    (gpEngFuncs.DxEngUnlockHdev.unwrap())((*peDdGl).hDev);
    InterlockedDecrement(core::ptr::addr_of_mut!((*peDdL).pobj.cExclusiveLock) as *mut LONG);

    RetVal
}
