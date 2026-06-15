//! win32ss/reactx/dxg/ddsurf.c

use win32k_sys::win32k::*;
use ntkrnl_sys::interlocked::InterlockedExchangeAdd;
use crate::types::*;
use crate::ddhmg::DdHmgLock;

#[no_mangle]
pub unsafe extern "system" fn DxDdLock(
    hSurface: HANDLE,
    puLockData: PDD_LOCKDATA,
    _hdcClip: HDC,
) -> DWORD {
    let pSurface: PEDD_SURFACE;
    let peDdL: PEDD_DIRECTDRAW_LOCAL;
    let peDdGl: PEDD_DIRECTDRAW_GLOBAL;
    let mut mapMemoryData: DD_MAPMEMORYDATA = core::mem::zeroed();

    pSurface = DdHmgLock(hSurface, ObjType_DDSURFACE_TYPE, 1) as PEDD_SURFACE;
    peDdL = (*pSurface).peDirectDrawLocal;
    peDdGl = (*peDdL).peDirectDrawGlobal2;

    if (*peDdL).isMemoryMapped == 0
        && ((*peDdGl).ddCallbacks.dwFlags & DDHAL_CB32_MAPMEMORY) != 0
    {
        mapMemoryData.bMap = 1;
        mapMemoryData.hProcess = (-1isize) as usize as HANDLE;
        mapMemoryData.fpProcess = 0;
        mapMemoryData.lpDD = peDdGl as PDD_DIRECTDRAW_GLOBAL;

        ((*peDdGl).ddCallbacks.MapMemory.unwrap())(&mut mapMemoryData);

        if mapMemoryData.ddRVal == 0 {
            (*peDdL).isMemoryMapped = 1;
            (*peDdL).fpProcess2 = mapMemoryData.fpProcess;
        }
    }

    if !pSurface.is_null() {
        InterlockedExchangeAdd(
            core::ptr::addr_of_mut!((*pSurface).pobj.cExclusiveLock) as *mut LONG,
            0xFFFFFFFFu32 as LONG,
        );
    }

    (*puLockData).ddRVal = DD_OK as i32;

    DDHAL_DRIVER_HANDLED
}

#[no_mangle]
pub unsafe extern "system" fn DxDdUnlock(
    _hSurface: HANDLE,
    puUnlockData: PDD_UNLOCKDATA,
) -> DWORD {
    (*puUnlockData).ddRVal = DD_OK as i32;

    DDHAL_DRIVER_HANDLED
}
